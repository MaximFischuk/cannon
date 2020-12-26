pub(crate) mod assert;
pub(crate) mod capture;
pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod executor;
pub(crate) mod hooks;
pub(crate) mod job;
pub(crate) mod operation;

use crate::app::context::Context;
use crate::app::context::ContextPool;
use crate::app::executor::JobExecutionHooks;
use crate::app::executor::JobGroup;
use crate::app::executor::RunInfo;
use crate::app::job::HttpJob;
use crate::configuration::manifest::Manifest;
use crate::configuration::manifest::ResourceType;
use crate::{app::capture::Capturable, configuration::manifest::JobType};
use crate::{
    app::capture::CaptureValue,
    configuration::manifest::Assertion,
    now,
    reporter::allure::model::{
        label::Label,
        stage::Stage,
        status::{Status, StatusDetails},
        test_result::{ExecutableItem, StepResult, TestResult},
    },
};
use liquid::Object;
use reqwest::blocking::Client;
use std::convert::TryInto;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc;

use self::{assert::Assertable, error::Error, operation::Performable};

macro_rules! lock {
    ($name: expr) => {
        match $name.lock() {
            Ok(locked) => locked,
            Err(e) => panic!("{:#?}", e),
        }
    };
}

#[derive(Debug)]
enum Event {
    GroupFinished(String),
    Reported(TestResult),
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
        let mut groups = Vec::with_capacity(manifest.pipeline.groups.len());
        for (group_name, job_list) in manifest.pipeline.groups {
            if !only.is_empty() && !only.contains(&group_name) {
                continue;
            }
            let mut http_jobs: Vec<(Box<dyn JobExecutionHooks + Send + Sync>, RunInfo)> =
                Vec::with_capacity(job_list.len());
            for entry in job_list {
                let info = RunInfo::new(
                    entry.name,
                    entry.repeats,
                    entry.delay,
                    entry.capture,
                    entry.on,
                    entry.assert,
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
        let (tx, mut rx) = mpsc::channel(2048);
        for jobs_group in groups {
            let context = self.context.clone();
            let sender = tx.clone();
            let mut reports = Vec::new();
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
                        let start_timestamp = now!();
                        let mut steps = vec![];
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
                        for assertion in &info.assertions {
                            let start_timestamp = now!();
                            let result = assertions(&local_context, assertion);
                            let step = StepResult::builder()
                                .name(local_context.apply(assertion.message.as_str()))
                                .status(
                                    result
                                        .map(|b| if b { Status::Passed } else { Status::Failed })
                                        .unwrap_or(Status::Broken),
                                )
                                .status_details(StatusDetails::from(
                                    "Test step 2 message".to_owned(),
                                ))
                                .stage(Stage::Finished)
                                .start(start_timestamp)
                                .stop(now!())
                                .build()
                                .unwrap();
                            steps.push(step);
                        }
                        let executable_item = ExecutableItem::builder()
                            .name(info.name.clone())
                            .status(Status::Passed)
                            .status_details(StatusDetails::from("Test description".to_owned()))
                            .start(start_timestamp)
                            .steps(steps)
                            .stop(now!())
                            .build()
                            .unwrap();
                        let result = TestResult::builder()
                            .item(executable_item)
                            .uuid(uuid::Uuid::new_v4())
                            .full_name(info.name.clone())
                            .test_case_id(info.id.clone())
                            .rerun_of(reports.first().map(TestResult::uuid))
                            .labels(vec![
                                Label::Language("English".to_owned()),
                                Label::Suite(jobs_group.name().clone()),
                            ])
                            .build()
                            .unwrap();
                        reports.push(result);
                    }
                    let mut locked_context = lock!(context);
                    locked_context.merge(local_context, jobs_group.name());
                    drop(locked_context);
                    // job.after(locked_context);
                }
                for report in reports {
                    sender
                        .send(Event::Reported(report))
                        .await
                        .expect("Incorrect sending event");
                }
                sender
                    .send(Event::GroupFinished(jobs_group.name()))
                    .await
                    .expect("Incorrect sending event");
            });
        }
        while let Some(event) = rx.recv().await {
            match event {
                Event::GroupFinished(group_name) => {
                    info!("Group {} done", group_name);
                    thread_count = thread_count - 1;
                    if thread_count == 0 {
                        break;
                    }
                }
                Event::Reported(report) => {
                    report.save_into_file("./allure-results").unwrap();
                }
            }
        }
    }
}

fn assertions(context: &Context, assertion: &Assertion) -> Result<bool, Error> {
    let result = assertion.assert.assert(&context);
    match result {
        Ok(result) => {
            info!(
                "{}...{}",
                context.apply(assertion.message.as_str()),
                if result { "ok" } else { "failed" }
            );
            Ok(result)
        }
        Err(e) => {
            error!("{:#?}", e);
            Err(e)
        }
    }
}
