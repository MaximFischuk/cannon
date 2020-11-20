use crate::app::error::Error as ExecutionError;
use crate::app::executor::ExecutionResponse;
use crate::app::CaptureValue;
use crate::app::Context;
use crate::app::GetUuid;
use crate::app::JobExecutionHooks;
use crate::configuration::manifest::PipelineEntry;
use crate::connection::SendMessage;
use bytes::Buf as BytesBuf;
use bytes::Bytes;
use http::Request as HttpRequest;
use http::Response as HttpResponse;
use reqwest::Error;
use reqwest::Method;
use std::collections::HashMap;
use std::time::Instant;

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

impl JobExecutionHooks<HttpRequest<Vec<u8>>, Result<HttpResponse<Bytes>, Error>> for HttpJob {
    fn before(&self, _context: &mut Context) -> Result<String, String> {
        todo!()
    }
    fn after(&self, _context: &mut Context) -> Result<String, String> {
        todo!()
    }
    fn execute(
        &self,
        context: &mut Context,
        sender: &impl SendMessage<HttpRequest<Vec<u8>>, Result<HttpResponse<Bytes>, Error>>,
    ) -> Result<ExecutionResponse, ExecutionError> {
        let mut request = HttpRequest::builder()
            .uri(context.apply(&self.request))
            .method(&self.method);
        for (key, value) in &self.headers {
            request = request.header(key, context.apply(&value));
        }
        let prepared;
        if let Some(body_data) = &self.body {
            let body = match String::from_utf8(body_data.bytes().to_vec()) {
                Ok(body) => context.apply(&body).as_bytes().to_vec(),
                Err(_) => body_data.bytes().to_vec(),
            };
            prepared = request.body(body).expect("Cannot create request");
        } else {
            prepared = request.body(Vec::default()).expect("Cannot create request");
        }
        let now = Instant::now();
        match sender.send(prepared) {
            Ok(response) => {
                let body = response.body();
                let elapsed = now.elapsed();
                debug!(
                    "Received response {:#?} body {:#?} in {} ms",
                    response,
                    body,
                    elapsed.as_millis()
                );
                let result = ExecutionResponse::builder()
                    .body(response.body().clone())
                    .execution_time(elapsed)
                    .additional(CaptureValue::default())
                    .build();
                result.map_err(|e| ExecutionError::Internal(e))
            }
            Err(e) => {
                error!("Failed to send request {}", e);
                Err(ExecutionError::Connection(format!("{}", e)))
            }
        }
    }
}
