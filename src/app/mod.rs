pub(crate) mod assert;
pub(crate) mod capture;
pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod executor;
pub(crate) mod hooks;
pub(crate) mod job;
pub(crate) mod operation;

use crate::app::capture::CaptureValue;
use crate::app::context::Context;
use crate::app::context::ContextPool;
use crate::app::executor::JobExecutionHooks;
use crate::app::executor::JobGroup;
use crate::app::executor::RunInfo;
use crate::app::job::HttpJob;
use crate::configuration::manifest::Manifest;
use crate::configuration::manifest::ResourceType;
use crate::{app::capture::Capturable, configuration::manifest::JobType};
use liquid::Object;
use reqwest::blocking::Client;
use std::convert::TryInto;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc;

use self::operation::Performable;

macro_rules! lock {
    ($name: expr) => {
        match $name.lock() {
            Ok(locked) => locked,
            Err(e) => panic!("{:#?}", e),
        }
    };
}

pub struct App {
    name: String,
    job_groups: Vec<JobGroup<Box<dyn JobExecutionHooks + Send + Sync>>>,
    context: Arc<Mutex<ContextPool>>,
}

impl App {
    pub fn new(manifest: Manifest, only: Vec<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        let client = Arc::new(client);
        let mut context = ContextPool::with_vars(manifest.vars);
        let mut groups = Vec::new();
        for (group_name, job_list) in manifest.pipeline.groups {
            if !only.is_empty() && !only.contains(&group_name) {
                continue;
            }
            let mut http_jobs: Vec<(Box<dyn JobExecutionHooks + Send + Sync>, RunInfo)> = vec![];
            for entry in job_list {
                let info = RunInfo::new(
                    entry.name,
                    entry.repeats,
                    entry.delay,
                    entry.capture,
                    entry.on,
                );
                context.push_contextual_vars(entry.vars, info.id);
                match entry.job_type {
                    JobType::Http {
                        request,
                        method,
                        headers,
                        body,
                    } => {
                        let job = HttpJob::new(
                            request,
                            method,
                            headers,
                            body.map(Into::into),
                            client.clone(),
                        );
                        http_jobs.push((Box::new(job), info));
                    }
                }
            }
            groups.push(JobGroup::new(group_name, http_jobs));
        }
        for resource in manifest.resources {
            match resource.r#type {
                ResourceType::File(path) => {
                    let path: Box<Path> = path.try_into().unwrap();
                    context.push_resource_file(resource.name, path);
                }
            }
        }
        App {
            name: manifest.name,
            job_groups: groups,
            context: Arc::new(Mutex::new(context)),
        }
    }

    pub async fn run(self) {
        info!("Starting pipeline '{}'", self.name);
        let groups = self.job_groups;
        let mut thread_count = groups.len();
        let (tx, mut rx) = mpsc::channel(100);
        for jobs_group in groups {
            let context = self.context.clone();
            let sender = tx.clone();
            tokio::spawn(async move {
                info!(
                    "Registered {} jobs in {} group",
                    jobs_group.amount(),
                    jobs_group.name()
                );
                for (job, info) in jobs_group.iter() {
                    let locked_context = lock!(context);
                    let mut local_context = locked_context.new_context(info.id);
                    drop(locked_context);
                    for i in 0..info.repeats {
                        debug!("Iteration {}", i);
                        if info.delay.gt(&Duration::default()) {
                            sleep(info.delay);
                        }
                        local_context.next();
                        // job.before(locked_context);
                        let now = Instant::now();
                        let result = job.execute(&local_context);
                        debug!(
                            "Elapsed for execution of test({}), {:?} ms",
                            info.id,
                            now.elapsed().as_millis()
                        );
                        let mut exported = Object::default();
                        match result {
                            Ok(body) => {
                                for entry in &info.captures {
                                    let value = entry.cap.capture(&local_context, body.body());
                                    debug!("Captured value: {:?}", value);
                                    exported.insert(entry.variable.clone().into(), value);
                                }
                                debug!(
                                    "Finished job '{}'({}) in {} ms",
                                    info.name,
                                    info.id,
                                    body.execution_time().as_millis()
                                );
                            }
                            Err(e) => error!("{:#?}", e),
                        }
                        local_context.push_vars(exported.clone());
                        for operation in &info.operations {
                            match operation.perform(&mut local_context) {
                                Ok(()) => {}
                                Err(e) => error!("{:#?}", e),
                            }
                        }
                    }
                    let mut locked_context = lock!(context);
                    locked_context.merge(local_context, jobs_group.name());
                    drop(locked_context);
                    // job.after(locked_context);
                }
                sender.send(jobs_group.name()).await.unwrap();
            });
        }
        while let Some(i) = rx.recv().await {
            info!("Group {} done", i);
            thread_count = thread_count - 1;
            if thread_count == 0 {
                break;
            }
        }
    }
}
