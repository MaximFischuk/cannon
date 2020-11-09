use std::time::Duration;
use bytes::Bytes;
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
use std::fs;
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

#[derive(Debug, Deserialize, Clone)]
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
pub enum AssertParamValueVar {
    Value(liquid::model::Value),
    Var(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssertFunction {
    Equal(AssertParamValueVar),
    NotEqual(AssertParamValueVar),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Functor {
    Assert {
        #[serde(flatten)]
        function: AssertFunction,

        #[serde(default)]
        message: Option<String>,
    },
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

#[derive(Debug, Deserialize, Default)]
pub struct Manifest {
    pub name: String,
    #[serde(with = "crate::configuration::deserialize::uri")]
    pub collect: Uri,
    pub pipeline: Pipeline,
    #[serde(default)]
    pub vars: Object,
}

#[derive(Debug, Deserialize, Default)]
pub struct Pipeline {
    pub before_all: Option<Code>,
    pub after_all: Option<Code>,
    pub test: Vec<PipelineEntry>,
}

// TODO: make unified or variant pipeline entry to support multiple client providers
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
    #[serde(with = "crate::configuration::deserialize::duration")]
    #[serde(default)]
    pub delay: Duration,
    #[serde(default = "default_repeats")]
    pub repeats: u64,
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

impl Into<Bytes> for BodyEntry {
    fn into(self) -> Bytes {
        match self {
            BodyEntry::Raw(body) => Bytes::from(body.to_owned()),
            BodyEntry::Json(body) => Bytes::from(serde_json::to_vec(&body).unwrap()),
            BodyEntry::Uri(body) => Bytes::from(read_uri(&body).unwrap()),
            BodyEntry::Base64(body) => Bytes::from(base64::decode(body).unwrap()),
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

fn default_repeats() -> u64 {
    1
}
