use std::sync::Arc;
use crate::app::CaptureValue;
use kstring::KString;
use liquid::Object;
use liquid::Parser;
use std::iter::FromIterator;
use derivative::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Context {
    globals: Object,
    contextual: Object,
    #[derivative(Debug="ignore")]
    parser: Arc<Parser>,
}

#[derive(Clone)]
pub struct Report {
    pub uuid: uuid::Uuid,
    pub duration: f64,
    pub status: u16,
}

impl Context {

    pub fn new() -> Self {
        Self {
            globals: Object::default(),
            contextual: Object::default(),
            parser: Arc::new(liquid::ParserBuilder::with_stdlib().build().unwrap()),
        }
    }

    pub fn with_vars<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (KString, CaptureValue)>,
    {
        let object = Object::from_iter(iter);
        Self {
            globals: object,
            contextual: Object::default(),
            parser: Arc::new(liquid::ParserBuilder::with_stdlib().build().unwrap()),
        }
    }

    pub fn push_vars<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (KString, CaptureValue)>,
    {
        self.globals.extend(iter);
    }

    pub fn push_contextual_vars<T>(&mut self, iter: T, local_id: uuid::Uuid)
    where
        T: IntoIterator<Item = (KString, CaptureValue)>
    {
        let object = Object::from_iter(iter);
        let key = local_id.to_simple().to_string().into();
        self.contextual.insert(key, CaptureValue::from(object));
    }

    pub fn apply(&self, body: &String) -> String {
        let template = match self.parser.parse(body.as_str()) {
            Ok(template) => template,
            Err(err) => panic!("Cannot unwind template, {:#?}", err),
        };
        match template.render(&self.globals) {
            Ok(rendered) => rendered,
            Err(err) => panic!("Cannot render template with data, {:#?}", err),
        }
    }

    pub fn find(&self, key: &String) -> Option<CaptureValue> {
        let k = KString::from(key.to_owned());
        self.globals.get(&k).map(Clone::clone)
    }

    pub fn isolated(&self, local_id: uuid::Uuid) -> Self {
        let mut contextual_vars = self.globals.clone();
        let key: KString = local_id.to_simple().to_string().into();
        let local = match self.contextual.get(&key) {
            Some(value) => value.clone().into_object().unwrap(),
            None => panic!("Local {} not found", local_id),
        };
        contextual_vars.extend(local);
        Self {
            globals: contextual_vars,
            contextual: Object::default(),
            parser: self.parser.clone()
        }
    }
}

#[cfg(test)]
mod test {

    use crate::app::Context;
    use crate::app::CaptureValue;
    
    #[test]
    fn test_apply_template() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let context = Context::with_vars(vec![("expect".into(), value.clone())]);
        let body = "{{expect}}".to_owned();
        let result = context.apply(&body);

        assert_eq!(result, "42".to_owned());
    }

}
