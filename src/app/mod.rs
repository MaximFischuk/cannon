use crate::configuration::manifest::BodyEntry;
use crate::configuration::manifest::Manifest;
use hyper::body::to_bytes;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper::{Body, Request};
use hyper_tls::HttpsConnector;
use liquid::Object;
use liquid::Parser;
use std::time::Instant;

pub struct App {
    manifest: Manifest,
    client: Client<HttpsConnector<HttpConnector>>,
    parser: Parser,
}

impl App {
    pub fn new(manifest: Manifest) -> Self {
        let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
        App {
            manifest,
            client,
            parser,
        }
    }

    pub async fn run(&self) {
        info!("Starting pipeline '{}'", self.manifest.name);
        for entry in &self.manifest.pipeline.test {
            info!("Test {}", entry.name);
            let mut request = Request::builder()
                .uri(entry.generate_request_uri())
                .method(&entry.method);
            for (key, value) in &entry.headers {
                request = request.header(key, value);
            }
            let prepared = request
                .body(App::build_body(
                    App::unwrap_body_entry(&entry.body)
                        .map(|body| self.apply_body_template(body, &entry.vars)),
                ))
                .expect("Cannot create request");
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

    fn apply_body_template(&self, body: String, values: &Object) -> String {
        let template = self.parser.parse(body.as_str()).unwrap();
        template.render(values).unwrap()
    }

    fn build_body(body: Option<String>) -> Body {
        match body {
            Some(body) => Body::from(body),
            None => Body::empty(),
        }
    }

    fn unwrap_body_entry(body_data: &Option<BodyEntry>) -> Option<String> {
        match body_data {
            Some(BodyEntry::Raw(body)) => Some(body.to_string()),
            Some(BodyEntry::Json(body)) => Some(serde_json::to_string(body).unwrap()),
            Some(BodyEntry::Uri(_body)) => None,
            None => None,
        }
    }
}
