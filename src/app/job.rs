use crate::app::executor::ExecutionResponse;
use crate::app::Context;
use crate::app::JobExecutionHooks;
use crate::connection::SendMessage;
use crate::{app::error::Error as ExecutionError, map};
use bytes::Buf as BytesBuf;
use bytes::Bytes;
use http::Request as HttpRequest;
use http::Response as HttpResponse;
use reqwest::Error;
use reqwest::Method;
use std::time::Instant;
use std::{collections::HashMap, sync::Arc};

use super::capture::Convert;

const HEADERS_KEY: &'static str = "headers";

pub struct HttpJob<T>
where
    T: SendMessage<HttpRequest<Vec<u8>>, Result<HttpResponse<Bytes>, Error>>,
{
    request: String,
    method: Method,
    headers: HashMap<String, String>,
    body: Option<Bytes>,
    client: Arc<T>,
}

impl<T> HttpJob<T>
where
    T: SendMessage<HttpRequest<Vec<u8>>, Result<HttpResponse<Bytes>, Error>>,
{
    pub fn new(
        request: String,
        method: Method,
        headers: HashMap<String, String>,
        body: Option<Bytes>,
        client: Arc<T>,
    ) -> HttpJob<T> {
        Self {
            request,
            method,
            headers,
            body,
            client,
        }
    }
}

impl<T> JobExecutionHooks for HttpJob<T>
where
    T: SendMessage<HttpRequest<Vec<u8>>, Result<HttpResponse<Bytes>, Error>>,
{
    fn before(&self, _context: &Context) -> Result<String, String> {
        todo!()
    }
    fn after(&self, _context: &Context) -> Result<String, String> {
        todo!()
    }
    fn execute(&self, context: &Context) -> Result<ExecutionResponse, ExecutionError> {
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
        match self.client.send(prepared) {
            Ok(response) => {
                let elapsed = now.elapsed();
                let body = response.body();
                debug!(
                    "Received response {:#?} body {:#?} in {} ms",
                    response,
                    body,
                    elapsed.as_millis()
                );
                let result = ExecutionResponse::builder()
                    .body(response.body().clone())
                    .execution_time(elapsed)
                    .additional(map! { HEADERS_KEY.to_owned() => response.headers().convert() })
                    .build();
                result.map_err(ExecutionError::Internal)
            }
            Err(e) => {
                error!("Failed to send request {}", e);
                Err(ExecutionError::Connection(format!("{}", e)))
            }
        }
    }
}
