use std::{fs::File, path::Path};

use regex::Regex;
use serde_derive::Serialize;
use serde_json::Error;

use super::{
    attachment::Attachment,
    label::Label,
    link::Link,
    parameter::Parameter,
    stage::Stage,
    status::{Status, StatusDetails},
};

pub type StepResult = ExecutableItem;
pub type FixtureResult = ExecutableItem;

#[derive(Debug, Serialize, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ExecutableItem {
    name: String,
    status: Status,
    status_details: StatusDetails,
    #[builder(default = "Stage::Finished")]
    stage: Stage,
    #[builder(default = "String::new()")]
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = "None")]
    description_html: Option<String>,
    #[builder(default = "Vec::new()")]
    steps: Vec<StepResult>,
    #[builder(default = "Vec::new()")]
    attachments: Vec<Attachment>,
    #[builder(default = "Vec::new()")]
    parameters: Vec<Parameter>,
    start: u128,
    stop: u128,
}

impl ExecutableItem {
    pub fn builder() -> ExecutableItemBuilder {
        ExecutableItemBuilder::default()
    }
}

#[derive(Debug, Serialize, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    #[serde(flatten)]
    item: ExecutableItem,
    uuid: uuid::Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = "None")]
    history_id: Option<uuid::Uuid>,
    full_name: String,
    test_case_id: uuid::Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = "None")]
    rerun_of: Option<uuid::Uuid>,
    #[builder(default = "Vec::new()")]
    labels: Vec<Label>,
    #[builder(default = "Vec::new()")]
    links: Vec<Link>,
}

impl TestResult {
    pub fn builder() -> TestResultBuilder {
        TestResultBuilder::default()
    }

    pub fn save_into_file(&self, path: &str) -> Result<(), Error> {
        std::fs::create_dir_all(Path::new(path)).unwrap();
        serde_json::to_writer(
            &File::create(format!("./allure-results/{}-result.json", self.uuid)).unwrap(),
            self,
        )
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestResultContainer {
    uuid: uuid::Uuid,
    name: String,
    children: Vec<uuid::Uuid>,
    description: String,
    description_html: String,
    befores: Vec<FixtureResult>,
    afters: Vec<FixtureResult>,
    links: Vec<Link>,
    start: u128,
    stop: u128,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorInfo {
    name: String,
    r#type: String,
    url: String,
    build_order: u64,
    build_name: String,
    build_url: String,
    report_url: String,
    report_name: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Blocker,
    Critical,
    Normal,
    Minor,
    Trivial,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Category {
    name: String,
    description: String,
    description_html: String,
    #[serde(with = "serde_regex")]
    message_regex: Regex,
    #[serde(with = "serde_regex")]
    trace_regex: Regex,
    matched_statuses: Vec<Status>,
    flaky: bool,
}
