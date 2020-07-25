use serde_json::Value;
use config::{Config, ConfigError, File};
use derivative::*;
use hyper::http::uri::Uri;
use jsonpath::Selector;
use serde::export::fmt::Debug;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceCode {
    Uri(#[serde(with = "crate::configuration::deserialize::uri")] Uri),
    Code(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Code {
    Js(SourceCode),
    Lua(SourceCode),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VarEntry {
    Single(String),
    Array(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BodyEntry {
    Raw(String),
    Json(Value),
    Uri(#[serde(with = "crate::configuration::deserialize::uri")] Uri),
}

#[derive(Deserialize, Derivative)]
#[serde(rename_all = "lowercase")]
#[derivative(Debug)]
pub enum Capture {
    Json(
        #[derivative(Debug = "ignore")]
        #[serde(with = "crate::configuration::deserialize::selector")]
        Selector,
    ),
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub name: String,
    #[serde(with = "crate::configuration::deserialize::uri")]
    pub collect: Uri,
    pub pipeline: Pipeline,
}

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub before_all: Option<Code>,
    pub after_all: Option<Code>,
    pub test: Vec<PipelineEntry>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineEntry {
    pub before: Option<Code>,
    pub after: Option<Code>,
    pub name: String,
    pub request: String,
    pub body: Option<BodyEntry>,
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    pub capture: Option<Capture>,
    // pub vars: HashMap<String, VarEntry>,
}

impl Manifest {
    pub fn from(file: PathBuf) -> Result<Self, ConfigError> {
        let mut config = Config::new();
        config
            .merge(File::from(file))
            .expect("Error while loading configuration from file");

        config.try_into()
    }
}

impl PipelineEntry {
    pub fn generate_request_uri(&self) -> Uri {
        let mut uri_string: String = self.request.clone();
        for (key, value) in &self.vars {
            uri_string = uri_string.replace(format!("{{{}}}", key).as_str(), value.as_str());
        }
        Uri::try_from(&uri_string).unwrap()
    }
}
