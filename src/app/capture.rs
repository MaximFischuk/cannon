use crate::app::context::Context;
use crate::configuration::manifest::AssertParamValueVar;
use crate::configuration::manifest::Capture;
use crate::configuration::manifest::CaptureEntry;
use bytes::Bytes;
use http::{HeaderMap, HeaderValue};
use liquid::Object;
use serde_json::Value;

pub type CaptureValue = liquid::model::Value;

pub(crate) trait Capturable<T> {
    fn capture(&self, ctx: &mut Context, data: &T) -> CaptureValue;
}

pub(crate) trait Resolvable {
    fn resolve(&self, ctx: &Context) -> Result<CaptureValue, String>;
}

pub trait Convert<T> {
    fn convert(&self) -> T;
}

impl Convert<CaptureValue> for Value {
    fn convert(&self) -> CaptureValue {
        match self {
            Value::Null => CaptureValue::Nil,
            Value::Number(num) if num.is_i64() => CaptureValue::scalar(num.as_i64().unwrap()),
            Value::Number(num) if num.is_u64() => {
                CaptureValue::scalar(num.as_u64().unwrap() as f64)
            }
            Value::Number(num) if num.is_f64() => CaptureValue::scalar(num.as_f64().unwrap()),
            Value::Bool(boolean) => CaptureValue::scalar(*boolean),
            Value::String(string) => CaptureValue::scalar(string.to_string()),
            Value::Array(array) => {
                CaptureValue::Array(array.iter().map(Value::convert).collect())
            }
            Value::Object(object) => {
                let mut liq_object = Object::new();
                for (key, value) in object {
                    liq_object.insert(key.clone().into(), value.convert());
                }
                CaptureValue::Object(liq_object)
            }
            _ => CaptureValue::Nil,
        }
    }
}

impl Convert<CaptureValue> for Vec<Value> {
    fn convert(&self) -> CaptureValue {
        if self.len() == 1 {
            self[0].convert()
        } else if !self.is_empty() {
            CaptureValue::Array(self.iter().map(Value::convert).collect())
        } else {
            CaptureValue::Nil
        }
    }
}

impl Convert<CaptureValue> for HeaderMap<HeaderValue> {
    fn convert(&self) -> CaptureValue {
        let mut object = Object::new();
        for (name, value) in self {
            let name_str = name.as_str();
            let value_str = value.as_bytes();
            object.insert(name_str.to_owned().into(), CaptureValue::scalar(String::from_utf8(Vec::from(value_str)).unwrap()));
        }
        CaptureValue::from(object)
    }
}

impl Capturable<Bytes> for &Vec<CaptureEntry> {
    fn capture(&self, _ctx: &mut Context, data: &Bytes) -> CaptureValue {
        let body_string = match String::from_utf8(data.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to parse body string: {}", e);
                return CaptureValue::default();
            }
        };
        let mut result = Object::new();
        for cap in *self {
            let value = match &cap.cap {
                Capture::Json(selector) => {
                    let data: Value = serde_json::from_str(&body_string)
                        .expect("Cannot serialize object to json");
                    let captured: Vec<Value> = selector.find(&data).cloned().collect();
                    captured.convert()
                }
                Capture::Regex(_) => unimplemented!(),
            };
            result.insert(cap.variable.clone().into(), value.clone());
        }
        CaptureValue::from(result)
    }
}

impl Capturable<Bytes> for Capture {
    fn capture(&self, _ctx: &mut Context, data: &Bytes) -> CaptureValue {
        let body_string = match String::from_utf8(data.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to parse body string: {}", e);
                return CaptureValue::default();
            }
        };
        let value = match self {
            Capture::Json(selector) => {
                let data: Value =
                    serde_json::from_str(&body_string).expect("Cannot serialize object to json");
                let captured: Vec<Value> = selector.find(&data).cloned().collect();
                captured.convert()
            }
            Capture::Regex(_) => unimplemented!(),
        };
        value
        // let result = Object::from_iter(once((self.variable.clone().into(), value.clone())));
        // CaptureValue::from(result)
    }
}

impl Capturable<String> for Capture {
    fn capture(&self, _ctx: &mut Context, data: &String) -> CaptureValue {
        let value = match self {
            Capture::Json(selector) => {
                let data: Value =
                    serde_json::from_str(data).expect("Cannot serialize object to json");
                let captured: Vec<Value> = selector.find(&data).cloned().collect();
                if captured.len() == 1 {
                    captured[0].convert()
                } else if !captured.is_empty() {
                    captured.convert()
                } else {
                    CaptureValue::Nil
                }
            }
            Capture::Regex(_) => unimplemented!(),
        };
        value
        // let result = Object::from_iter(once((self.variable.clone().into(), value.clone())));
        // CaptureValue::from(result)
    }
}

impl Resolvable for AssertParamValueVar {
    fn resolve(&self, ctx: &Context) -> Result<CaptureValue, String> {
        match self {
            AssertParamValueVar::Value(object) => Ok(object.clone()),
            AssertParamValueVar::Var(var_name) => match ctx.find(var_name) {
                Some(value) => Ok(value),
                None => Err("Value not found".to_owned()),
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_json::json;

    #[test]
    fn test_convertation_into_liquid_value() {
        let value_null = json!(null);
        let value_number_int = json!(42);
        let value_number_float = json!(42.5);
        let value_bool = json!(true);
        let value_string = json!("some string");
        let value_array = json!(["an", "array"]);
        let value_object = json!({ "an": "object" });

        assert!(value_null.convert().as_view().is_nil());
        {
            let value = value_number_int.convert();
            assert_eq!(value, CaptureValue::scalar(42));
        }
        {
            let value = value_number_float.convert();
            assert_eq!(value, CaptureValue::scalar(42.5));
        }
        {
            let value = value_bool.convert();
            assert_eq!(value, CaptureValue::scalar(true));
        }
        {
            let value = value_string.convert();
            assert_eq!(value, CaptureValue::scalar("some string"));
        }
        {
            let value = value_array.convert();
            let expected = CaptureValue::Array(vec![
                CaptureValue::scalar("an"),
                CaptureValue::scalar("array"),
            ]);
            assert_eq!(value, expected);
        }
        {
            let value = value_object.convert();
            let object: Object = [("an".into(), CaptureValue::scalar("object"))]
                .iter()
                .cloned()
                .collect();
            let expected = CaptureValue::Object(object);
            assert_eq!(value, expected);
        }
    }
}
