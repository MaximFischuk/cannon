use config::{Config, ConfigError, File};
use derivative::*;
use hyper::http::uri::Uri;
use hyper::Method;
use jsonpath::Selector;
use liquid::Object;
use regex::Regex;
use serde::export::fmt::Debug;
use serde_derive::Deserialize;
use serde_json::Value;
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
    Base64(#[serde(with = "crate::configuration::deserialize::base64_property")] Vec<u8>),
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    #[serde(with = "crate::configuration::deserialize::uri")]
    pub uri: Uri,
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
    Regex(#[serde(with = "serde_regex")] Regex),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssertFunction {
    Equal(String, String, #[serde(default)] Option<String>),
    NotEqual(String, String, #[serde(default)] Option<String>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Functor {
    Assert(AssertFunction),
    Matches(#[serde(with = "serde_regex")] Regex),
}

#[derive(Debug, Deserialize)]
pub struct CaptureEntry {
    #[serde(flatten)]
    pub cap: Capture,
    #[serde(rename = "as")]
    pub variable: String,
    pub on: Vec<Functor>,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub name: String,
    #[serde(with = "crate::configuration::deserialize::uri")]
    pub collect: Uri,
    pub pipeline: Pipeline,
    #[serde(default)]
    pub vars: Object,
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
    #[serde(with = "crate::configuration::deserialize::http_method")]
    #[serde(default)]
    pub method: Method,
    pub body: Option<BodyEntry>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub vars: Object,
    #[serde(default)]
    pub capture: Vec<CaptureEntry>,
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
        let uri_string: String = self.request.clone();
        let template = liquid::ParserBuilder::with_stdlib()
            .build()
            .unwrap()
            .parse(uri_string.as_str())
            .unwrap();
        Uri::try_from(&template.render(&self.vars).unwrap()).unwrap()
    }
}
