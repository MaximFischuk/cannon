use bytes::Bytes;
use config::{Config, ConfigError, File};
use derivative::*;
use http::Uri;
use jsonpath::Selector;
use liquid::Object;
use regex::Regex;
use reqwest::Method;
use serde::{export::fmt::Debug, Deserializer};
use serde_derive::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

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
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    File(PathBuf),
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    #[serde(flatten)]
    pub r#type: ResourceType,
    pub name: String,
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

#[derive(Debug)]
pub enum Variable {
    Value(liquid::model::Value),
    Template(String),
    Path(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssertFunction {
    Equal(Variable, Variable),
    NotEqual(Variable, Variable),
    Matches(Variable, #[serde(with = "serde_regex")] Regex),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Assertion {
    pub message: String,

    #[serde(flatten)]
    pub assert: AssertFunction,
}

#[derive(Debug, Deserialize)]
pub struct CaptureEntry {
    #[serde(flatten)]
    pub cap: Capture,

    #[serde(rename = "as")]
    pub variable: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Add(String, liquid::model::Value),
    PushCsv(String, PathBuf),
    Console(String),
}

#[derive(Debug, Deserialize, Default)]
pub struct Manifest {
    pub name: String,

    #[serde(with = "crate::configuration::deserialize::uri")]
    pub collect: Uri,

    pub pipeline: Pipeline,

    #[serde(default)]
    pub vars: Object,

    #[serde(default)]
    pub resources: Vec<Resource>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Pipeline {
    pub before_all: Option<Code>,

    pub after_all: Option<Code>,

    #[serde(flatten)]
    pub groups: HashMap<String, Vec<PipelineEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum JobType {
    Http {
        request: String,

        #[serde(with = "crate::configuration::deserialize::http_method")]
        #[serde(default)]
        method: Method,

        body: Option<BodyEntry>,

        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct PipelineEntry {
    pub before: Option<Code>,

    pub after: Option<Code>,

    pub name: String,

    #[serde(flatten)]
    pub job_type: JobType,

    #[serde(default)]
    pub vars: Object,

    #[serde(default)]
    pub capture: Vec<CaptureEntry>,

    #[serde(default)]
    pub on: Vec<Operation>,

    #[serde(default)]
    pub assert: Vec<Assertion>,

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
            BodyEntry::Raw(body) => Bytes::from(body),
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

impl<'de> serde::de::Deserialize<'de> for Variable {
    fn deserialize<D>(deserializer: D) -> Result<Variable, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = liquid::model::Value::deserialize(deserializer)?;
        if let Some(value_scalar) = value.clone().into_scalar() {
            let value_string = value_scalar.into_string();
            if value_string.contains("{{") {
                return Ok(Variable::Template(value_string.into_string()));
            } else if value_string.contains(".") {
                let splited = value_string.split(".");
                return Ok(Variable::Path(splited.map(str::to_owned).collect()));
            }
        }
        Ok(Variable::Value(value))
    }
}
