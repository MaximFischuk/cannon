use crate::configuration::manifest::BodyEntry;
use crate::configuration::manifest::Manifest;
use hyper::body::to_bytes;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper::{Body, Method, Request};
use hyper_tls::HttpsConnector;
use std::time::Instant;

pub struct App {
    manifest: Manifest,
    client: Client<HttpsConnector<HttpConnector>>,
}

impl App {
    pub fn new(manifest: Manifest) -> Self {
        let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        App { manifest, client }
    }

    pub async fn run(&self) {
        info!("Starting pipeline '{}'", self.manifest.name);
        for entry in &self.manifest.pipeline.test {
            info!("Test {}", entry.name);
            let method = entry
                .headers
                .get(&"Method".to_owned())
                .map(String::as_str)
                .unwrap_or(&Method::GET.as_str());
            let mut request = Request::builder()
                .uri(entry.generate_request_uri())
                .method(method);
            for (key, value) in &entry.headers {
                request = request.header(key, value);
            }
            let prepared = request.body(App::build_body(&entry.body)).expect("Cannot create request");
            let now = Instant::now();
            match self.client.request(prepared).await {
                Ok(mut response) => {
                    let body = to_bytes(response.body_mut()).await;
                    info!(
                        "Received response {:#?} body {:#?} in {} ms",
                        response,
                        body,
                        now.elapsed().as_millis()
                    )
                }
                Err(e) => error!("Failed to send request {}", e),
            }
        }
    }

    fn build_body(body_data: &Option<BodyEntry>) -> Body {
        match body_data {
            Some(BodyEntry::Raw(body)) => Body::from(body.to_string()),
            Some(BodyEntry::Uri(_body)) => Body::empty(),
            None => Body::empty()
        }
    }
}
