use hyper::body::to_bytes;
use futures::executor::block_on;
use std::time::Instant;
use crate::app::SendMessage;
use crate::app::Context;
use hyper::client::ResponseFuture;
use hyper::Body;
use hyper::Request;
use crate::app::JobExecutionHooks;
use crate::app::GetUuid;
use crate::configuration::manifest::PipelineEntry;
use bytes::Bytes;
use std::collections::HashMap;
use hyper::Method;
use bytes::Buf as BytesBuf;
use hyper::body::Buf;

pub struct HttpJob {
    id: uuid::Uuid,
    name: String,
    request: String,
    method: Method,
    headers: HashMap<String, String>,
    body: Option<Bytes>,
}

impl GetUuid for HttpJob {
    fn get_uuid(&self) -> uuid::Uuid {
        self.id
    }
}

impl From<&PipelineEntry> for HttpJob {
    fn from(entry: &PipelineEntry) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: entry.name.clone(),
            request: entry.request.clone(),
            method: entry.method.clone(),
            headers: entry.headers.clone(),
            body: entry.body.clone().map(Into::into),
        }
    }
}

impl JobExecutionHooks<Request<Body>, ResponseFuture> for HttpJob {
    fn before(&self, _context: &mut Context) -> Result<String, String> {
        todo!()
    }
    fn after(&self, _context: &mut Context) -> Result<String, String> {
        todo!()
    }
    fn execute(
        &self,
        context: &mut Context,
        sender: &impl SendMessage<Request<Body>, ResponseFuture>,
    ) -> Result<Bytes, String> {
        let mut request = Request::builder()
            .uri(context.apply(&self.request))
            .method(&self.method);
        for (key, value) in &self.headers {
            request = request.header(key, context.apply(&value));
        }
        let prepared;
        if let Some(body_data) = &self.body {
            let body = match String::from_utf8(body_data.bytes().to_vec()) {
                Ok(body) => Body::from(context.apply(&body)),
                Err(_) => Body::from(body_data.bytes().to_vec()),
            };
            prepared = request.body(body).expect("Cannot create request");
        } else {
            prepared = request.body(Body::empty()).expect("Cannot create request");
        }
        let now = Instant::now();
        match block_on(sender.send(prepared)) {
            Ok(mut response) => {
                let body = block_on(to_bytes(response.body_mut())).unwrap();
                info!(
                    "Received response {:#?} body {:#?} in {} ms",
                    response,
                    body,
                    now.elapsed().as_millis()
                );
                Ok(Bytes::copy_from_slice(body.bytes()))
            }
            Err(e) => {
                error!("Failed to send request {}", e);
                Err(format!("{}", e))
            },
        }
    }
}