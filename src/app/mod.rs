pub(crate) mod capture;
pub(crate) mod context;
pub(crate) mod executor;
pub(crate) mod hooks;
pub(crate) mod assert;
pub(crate) mod job;

use crate::app::job::HttpJob;
use crate::configuration::manifest::CaptureEntry;
use crate::app::executor::JobGroup;
use std::collections::HashMap;
use std::time::Instant;
use crate::app::capture::Capturable;
use crate::app::capture::CaptureValue;
use crate::app::context::Context;
use crate::app::executor::SendMessage;
use crate::app::executor::{JobExecutionHooks, GetUuid};
use crate::configuration::manifest::Manifest;
use crate::app::assert::Assertable;
use hyper::client::HttpConnector;
use hyper::client::ResponseFuture;
use hyper::Body;
use hyper::Client;
use hyper::Request;
use hyper_tls::HttpsConnector;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;
use std::iter::once;

impl SendMessage<Request<Body>, ResponseFuture> for Client<HttpsConnector<HttpConnector>> {
    fn send(&self, data: Request<Body>) -> ResponseFuture {
        self.request(data)
    }
}

pub struct App {
    name: String,
    jobs_group: JobGroup<HttpJob>,
    captures: HashMap<uuid::Uuid, Vec<CaptureEntry>>,
    client: Client<HttpsConnector<HttpConnector>>,
    context: Arc<Mutex<Context>>,
}

impl App {
    pub fn new(manifest: Manifest) -> Self {
        let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let mut context = Context::with_vars(manifest.vars);
        let mut http_jobs = vec![];
        let mut captures = HashMap::new();
        for entry in manifest.pipeline.test {
            let job = HttpJob::from(&entry);
            if !entry.capture.is_empty() {
                captures.insert(job.get_uuid(), entry.capture);
            }
            context.push_contextual_vars(entry.vars, job.get_uuid());
            http_jobs.push(Box::new(job));
        }
        App {
            client,
            captures,
            name: manifest.name,
            jobs_group: JobGroup::new(String::default(), http_jobs),
            context: Arc::new(Mutex::new(context)),
        }
    }

    pub async fn run(&self) {
        info!("Starting pipeline '{}'", self.name);
        info!("Registered {} jobs", self.jobs_group.amount());
        for job in self.jobs_group.iter() {
            let mut lock = match self.context.lock() {
                Ok(lock) => lock,
                Err(e) => panic!("{:#?}", e),
            };
            let locked_context = lock.deref_mut();
            let local_context = &mut locked_context.make_contextual(job.get_uuid());
            // job.before(locked_context);
            let now = Instant::now();
            let result = job.execute(local_context, &self.client);
            info!("Elapsed for execution of test({}), {:?} ms", job.get_uuid(), now.elapsed().as_millis());
            match result {
                Ok(body) => {
                    let captures = self.captures.get(&job.get_uuid());
                    if let Some(cap) = captures {
                        for entry in cap {
                            let mut assert_result: bool = true;
                            let value = entry.cap.capture(local_context, &body);
                            for functor in &entry.on {
                                assert_result &= functor.assert(local_context, &value);
                            }
                            info!("Captured value: {:?}", value);
                            if assert_result {
                                locked_context.push_vars(once((entry.variable.clone().into(), value)));
                            }
                        }
                    }
                },
                Err(e) => error!("{:#?}", e)
            }
            // job.after(locked_context);
            drop(lock);
        }
    }
}
