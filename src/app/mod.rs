use crate::configuration::manifest::AssertFunction;
use crate::configuration::manifest::AssertParamValueVar;
use crate::configuration::manifest::BodyEntry;
use crate::configuration::manifest::Functor;
use crate::configuration::manifest::Manifest;
use crate::configuration::manifest::{Capture, CaptureEntry};
use hyper::body::to_bytes;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper::Uri;
use hyper::{Body, Request};
use hyper_tls::HttpsConnector;
use kstring::KString;
use liquid::Object;
use liquid::Parser;
use serde_json::Value;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use liquid::model::Value as LqValue;

pub struct App {
    manifest: Manifest,
    client: Client<HttpsConnector<HttpConnector>>,
    parser: Parser,
    globals: Arc<Mutex<Object>>,
}

impl App {
    pub fn new(manifest: Manifest) -> Self {
        let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
        App {
            manifest,
            client,
            parser,
            globals: Arc::default(),
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
                    let body = to_bytes(response.body_mut()).await.unwrap();
                    debug!(
                        "Received response {:#?} body {:#?} in {} ms",
                        response,
                        body,
                        now.elapsed().as_millis()
                    );
                    self.capture_body(&body, &entry.capture);
                }
                Err(e) => error!("Failed to send request {}", e),
            }
        }
    }

    fn capture_body(&self, body: &[u8], capture: &[CaptureEntry]) -> Object {
        let body_string = match String::from_utf8(body.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to parse body string: {}", e);
                return Object::default();
            }
        };
        let mut result = Object::new();
        for cap in capture {
            let value = match &cap.cap {
                Capture::Json(selector) => {
                    let data: Value = serde_json::from_str(&body_string)
                        .expect("Cannot serialize object to json");
                    let captured: Vec<Value> = selector.find(&data).cloned().collect();
                    if captured.len() == 1 {
                        captured[0].into_liquid()
                    } else if !captured.is_empty() {
                        captured.into_liquid()
                    } else {
                        LqValue::Nil
                    }
                }
                Capture::Regex(_) => unimplemented!(),
            };
            let mut assert_result;
            for assertion in &cap.on {
                match assertion {
                    Functor::Assert { function, message } => {
                        assert_result = self.assert_value(&value, function);
                        trace!("Assert result {}", assert_result);
                        if !assert_result {
                            if let Some(message) = message {
                                info!("Assertation failed: {}", message);
                            }
                        }
                    }
                    Functor::Matches(_pattern) => {
                        unimplemented!();
                    }
                }
                if assert_result {
                    result.insert(cap.variable.clone().into(), value.clone());
                }
            }
        }
        result
    }

    fn resolve_assert_parameter(&self, value: &AssertParamValueVar) -> Result<LqValue, String> {
        trace!("Resolving assert parameter {:?}", value);
        match value {
            AssertParamValueVar::Value(object) => Ok(object.clone()),
            AssertParamValueVar::Var(var_name) => {
                let key = KString::from(var_name.clone());
                let lock = self.globals.lock().unwrap();
                if let Some(value) = (*lock).get(&key) {
                    return Ok(value.clone());
                }
                drop(lock);
                match self.manifest.vars.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Err(String::from("Value not found")),
                }
            }
        }
    }

    fn assert_value(&self, value: &LqValue, assert: &AssertFunction) -> bool {
        trace!("Assertation value: {:#?} to {:#?}", value, assert);
        match assert {
            AssertFunction::Equal(var) => match self.resolve_assert_parameter(var) {
                Ok(expected) => {
                    trace!("Check equals of {:?} to {:?}", expected, value);
                    value == &expected
                }
                Err(e) => {
                    error!("{}", e);
                    false
                }
            },
            AssertFunction::NotEqual(var) => match self.resolve_assert_parameter(var) {
                Ok(expected) => value != &expected,
                Err(e) => {
                    error!("{}", e);
                    false
                }
            },
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

// TODO: move this to a separate module
trait IntoLiquid<T> {
    fn into_liquid(&self) -> T;
}

impl IntoLiquid<LqValue> for Value {
    fn into_liquid(&self) -> LqValue {
        match self {
            Value::Null => LqValue::Nil,
            Value::Number(num) if num.is_i64() => LqValue::scalar(num.as_i64().unwrap()),
            Value::Number(num) if num.is_u64() => LqValue::scalar(num.as_u64().unwrap() as f64),
            Value::Number(num) if num.is_f64() => LqValue::scalar(num.as_f64().unwrap()),
            Value::Bool(boolean) => LqValue::scalar(*boolean),
            Value::String(string) => LqValue::scalar(string.to_string()),
            Value::Array(array) => LqValue::Array(array.iter().map(Value::into_liquid).collect()),
            Value::Object(object) => {
                let mut liq_object = Object::new();
                for (key, value) in object {
                    liq_object.insert(key.clone().into(), value.into_liquid());
                }
                LqValue::Object(liq_object)
            }
            _ => LqValue::Nil,
        }
    }
}

impl IntoLiquid<LqValue> for Vec<Value> {
    fn into_liquid(&self) -> LqValue {
        LqValue::Array(self.iter().map(Value::into_liquid).collect())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_json::json;
    use std::iter::FromIterator;

    #[test]
    fn test_value_equals_to_expexted_value() {
        let manifest = Manifest::default();
        let app = App::new(manifest);
        let value = LqValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::Equal(AssertParamValueVar::Value(LqValue::Scalar(
            liquid::model::scalar::Scalar::new(42),
        )));
        let result = app.assert_value(&value, &assert_function);

        assert!(result);
    }

    #[test]
    fn test_value_equals_to_variable() {
        let value = LqValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let object = Object::from_iter(vec![("expect".into(), value)]);
        let mut manifest = Manifest::default();
        manifest.vars = object;
        let app = App::new(manifest);
        let value = LqValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::Equal(AssertParamValueVar::Var("expect".into()));
        let result = app.assert_value(&value, &assert_function);

        assert!(result);
    }

    #[test]
    fn test_value_not_equals_to_expexted_value() {
        let manifest = Manifest::default();
        let app = App::new(manifest);
        let value = LqValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::NotEqual(AssertParamValueVar::Value(
            LqValue::Scalar(liquid::model::scalar::Scalar::new(43)),
        ));
        let result = app.assert_value(&value, &assert_function);

        assert!(result);
    }

    #[test]
    fn test_value_not_equals_to_variable() {
        let value = LqValue::Scalar(liquid::model::scalar::Scalar::new(43));
        let object = Object::from_iter(vec![("expect".into(), value)]);
        let mut manifest = Manifest::default();
        manifest.vars = object;
        let app = App::new(manifest);
        let value = LqValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::NotEqual(AssertParamValueVar::Var("expect".into()));
        let result = app.assert_value(&value, &assert_function);

        assert!(result);
    }

    #[test]
    fn test_convertation_into_liquid_value() {
        let value_null = json!(null);
        let value_number_int = json!(42);
        let value_number_float = json!(42.5);
        let value_bool = json!(true);
        let value_string = json!("some string");
        let value_array = json!(["an", "array"]);
        let value_object = json!({ "an": "object" });

        assert!(value_null.into_liquid().as_view().is_nil());
        {
            let value = value_number_int.into_liquid();
            assert_eq!(value, LqValue::scalar(42));
        }
        {
            let value = value_number_float.into_liquid();
            assert_eq!(value, LqValue::scalar(42.5));
        }
        {
            let value = value_bool.into_liquid();
            assert_eq!(value, LqValue::scalar(true));
        }
        {
            let value = value_string.into_liquid();
            assert_eq!(value, LqValue::scalar("some string"));
        }
        {
            let value = value_array.into_liquid();
            let expected = LqValue::Array(vec![LqValue::scalar("an"), LqValue::scalar("array")]);
            assert_eq!(value, expected);
        }
        {
            let value = value_object.into_liquid();
            let object: Object = [("an".into(), LqValue::scalar("object"))]
                .iter()
                .cloned()
                .collect();
            let expected = LqValue::Object(object);
            assert_eq!(value, expected);
        }
    }
}
