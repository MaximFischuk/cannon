use crate::app::CaptureValue;
use csv::{StringRecordsIntoIter, Reader};
use derivative::*;
use kstring::KString;
use liquid::{Object, model::Value};
use liquid::Parser;
use std::{collections::HashMap, path::Path};
use std::fs::File;
use std::iter::FromIterator;
use std::sync::Arc;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Context {
    #[derivative(Debug = "ignore")]
    parser: Arc<Parser>,
    variables: Object,
    #[derivative(Debug = "ignore")]
    records: HashMap<String, StringRecordsIntoIter<File>>,
}

#[derive(Clone)]
pub struct Report {
    pub uuid: uuid::Uuid,
    pub duration: f64,
    pub status: u16,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(in crate::app) struct ContextPool {
    globals: Object,
    contextual: Object,
    #[derivative(Debug = "ignore")]
    parser: Arc<Parser>,
    resources: HashMap<String, Box<Path>>,
}

impl ContextPool {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            globals: Object::default(),
            contextual: Object::default(),
            parser: Arc::new(liquid::ParserBuilder::with_stdlib().build().unwrap()),
            resources: HashMap::new(),
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
            resources: HashMap::new(),
        }
    }

    pub fn push_vars<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (KString, CaptureValue)>,
    {
        self.globals.extend(iter);
    }

    pub fn push_resource_file(&mut self, name: String, path: Box<Path>) {
        self.resources.insert(name, path);
    }

    pub fn push_contextual_vars<T>(&mut self, iter: T, local_id: uuid::Uuid)
    where
        T: IntoIterator<Item = (KString, CaptureValue)>,
    {
        let object = Object::from_iter(iter);
        let key = local_id.to_simple().to_string().into();
        self.contextual.insert(key, CaptureValue::from(object));
    }

    #[inline]
    #[cfg(test)]
    pub fn default_context(&self) -> Context {
        self.new_context(uuid::Uuid::default())
    }

    pub fn new_context(&self, local_id: uuid::Uuid) -> Context {
        let mut contextual_vars = self.globals.clone();
        let key: KString = local_id.to_simple().to_string().into();
        if let Some(value) = self.contextual.get(&key) {
            let local = value.clone().into_object().unwrap();
            contextual_vars.extend(local);
        }
        let mut records = HashMap::new();
        for (key, value) in &self.resources {
            match Reader::from_path(value) {
                Ok(reader) => {
                    records.insert(key.to_owned(), reader.into_records());
                },
                Err(err) => {
                    error!("Cannot create resource reader '{}'", err);
                }
            };
        }
        Context {
            variables: contextual_vars,
            parser: self.parser.clone(),
            records,
        }
    }
}

impl Context {
    pub fn apply(&self, body: &str) -> String {
        let template = match self.parser.parse(body) {
            Ok(template) => template,
            Err(err) => panic!("Cannot unwind template, {:#?}", err),
        };
        match template.render(&self.variables) {
            Ok(rendered) => rendered,
            Err(err) => panic!("Cannot render template with data, {:#?}", err),
        }
    }

    pub fn find(&self, key: &str) -> Option<CaptureValue> {
        let k = KString::from(key.to_owned());
        self.variables.get(&k).map(Clone::clone)
    }

    pub fn next(&mut self) {
        for (key, record) in &mut self.records {
            let headers = record.reader_mut().headers().unwrap().to_owned();
            match record.next() {
                Some(Ok(value)) => {
                    let mut object = Object::new();
                    for pos in 0..headers.len() {
                        if let (Some(h), Some(v)) = (headers.get(pos), value.get(pos)) {
                            object.insert(h.to_owned().into(), Value::scalar(v.to_owned()));
                        }
                    }
                    self.variables.insert(key.to_owned().into(), CaptureValue::from(object));
                },
                Some(Err(err)) => {
                    error!("Cannot unwind variable {:?}", err);
                },
                None => {
                    error!("No iteration data remains");
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::ContextPool;
    use crate::app::CaptureValue;

    #[test]
    fn test_apply_template() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let context =
            ContextPool::with_vars(vec![("expect".into(), value)]).default_context();
        let body = "{{expect}}".to_owned();
        let result = context.apply(&body);

        assert_eq!(result, "42".to_owned());
    }
}
