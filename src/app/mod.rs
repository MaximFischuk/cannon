pub(crate) mod assert;
pub(crate) mod capture;
pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod executor;
pub(crate) mod hooks;
pub(crate) mod job;

use crate::app::assert::Assertable;
use crate::app::capture::Capturable;
use crate::app::capture::CaptureValue;
use crate::app::context::Context;
use crate::app::context::ContextPool;
use crate::app::executor::JobGroup;
use crate::app::executor::RunInfo;
use crate::app::executor::{GetUuid, JobExecutionHooks};
use crate::app::job::HttpJob;
use crate::configuration::manifest::Manifest;
use liquid::Object;
use reqwest::blocking::Client;
use std::convert::TryInto;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

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
    jobs_group: JobGroup<HttpJob>,
    client: Client,
    context: Arc<Mutex<ContextPool>>,
}

impl App {
    pub fn new(manifest: Manifest) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        let mut context = ContextPool::with_vars(manifest.vars);
        let mut http_jobs = vec![];
        for entry in manifest.pipeline.test {
            let job = HttpJob::from(&entry);
            let info = RunInfo::new(entry.repeats, entry.delay, entry.capture);
            context.push_contextual_vars(entry.vars, job.get_uuid());
            http_jobs.push((job, info));
        }
        for resource in manifest.resources {
            let path: Box<Path> = resource.try_into().unwrap();
            match csv::Reader::from_path(&path) {
                Ok(reader) => context.push_csv_file_reader(
                    path.file_stem().unwrap().to_str().unwrap().to_owned(),
                    reader,
                ),
                Err(err) => error!("Cannot create resource reader cause: {}", err),
            };
        }
        App {
            client,
            name: manifest.name,
            jobs_group: JobGroup::new(String::default(), http_jobs),
            context: Arc::new(Mutex::new(context)),
        }
    }

    pub fn run(&self) {
        info!("Starting pipeline '{}'", self.name);
        info!("Registered {} jobs", self.jobs_group.amount());
        for (job, info) in self.jobs_group.iter() {
            let locked_context = lock!(self.context);
            let mut local_context = locked_context.new_context(job.get_uuid());
            drop(locked_context);
            let mut exported = Object::default();
            for i in 0..info.repeats {
                info!("Iteration {}", i);
                sleep(info.delay);
                // job.before(locked_context);
                let now = Instant::now();
                let result = job.execute(&mut local_context, &self.client);
                info!(
                    "Elapsed for execution of test({}), {:?} ms",
                    job.get_uuid(),
                    now.elapsed().as_millis()
                );
                match result {
                    Ok(body) => {
                        for entry in &info.captures {
                            let mut assert_result: bool = true;
                            let value = entry.cap.capture(&mut local_context, body.body());
                            for functor in &entry.on {
                                assert_result &= functor.assert(&mut local_context, &value);
                            }
                            info!("Captured value: {:?}", value);
                            if assert_result {
                                exported.insert(entry.variable.clone().into(), value);
                            }
                        }
                    }
                    Err(e) => error!("{:#?}", e),
                }
            }
            let mut locked_context = lock!(self.context);
            locked_context.push_vars(exported);
            // job.after(locked_context);
        }
    }
}
