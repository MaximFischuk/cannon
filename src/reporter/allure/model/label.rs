use serde_derive::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "name", content = "value")]
pub enum Label {
    #[serde(rename = "AS_ID")]
    AsId,
    Suite(String),
    ParentSuite,
    SubSuite,
    Epic,
    Feature,
    Story,
    Severity,
    Tag,
    Owner,
    Lead,
    Host,
    Thread,
    TestMethod,
    TestClass,
    Package,
    Framework,
    Language(String),
    // #[serde(untagged)]
    // Custom(String),
}
