use crate::configuration::manifest::BodyEntry;
use crate::configuration::manifest::Manifest;
use hyper::body::to_bytes;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper::Uri;
use hyper::{Body, Request};
use hyper_tls::HttpsConnector;
use liquid::Object;
use liquid::Parser;
use std::fs;
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
                request = request.header(
                    key,
                    self.apply_body_template(value.to_string(), &entry.vars),
                );
            }
            let prepared;
            if let Some(body_data) = App::unwrap_body_entry(&entry.body) {
                let body = match String::from_utf8(body_data.clone()) {
                    Ok(body) => Body::from(self.apply_body_template(body, &entry.vars)),
                    Err(_) => Body::from(body_data),
                };
                prepared = request.body(body).expect("Cannot create request");
            } else {
                prepared = request.body(Body::empty()).expect("Cannot create request");
            }
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

    fn unwrap_body_entry(body_data: &Option<BodyEntry>) -> Option<Vec<u8>> {
        match body_data {
            Some(BodyEntry::Raw(body)) => Some(Vec::from(body.as_bytes())),
            Some(BodyEntry::Json(body)) => Some(serde_json::to_vec(body).unwrap()),
            Some(BodyEntry::Uri(body)) => read_uri(body),
            Some(BodyEntry::Base64(body)) => Some(base64::decode(body).unwrap()),
            None => None,
        }
    }
}

fn read_uri(uri: &Uri) -> Option<Vec<u8>> {
    if let Some(scheme) = uri.scheme_str() {
        return match scheme {
            "file" => match fs::read(format!("{}{}", uri.authority().unwrap(), uri.path())) {
                Ok(file_data) => Some(file_data),
                Err(e) => {
                    error!(
                        "Failed to load file content from {} cause {}",
                        uri.path(),
                        e
                    );
                    None
                }
            },
            _ => None,
        };
    }
    None
}
