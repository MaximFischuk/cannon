use serde_derive::Serialize;

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusDetails {
    // known: bool,
    // muted: bool,
    // flaky: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<String>,
}

impl From<String> for StatusDetails {
    fn from(message: String) -> Self {
        Self {
            message,
            trace: None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Failed,
    Broken,
    Passed,
    Skipped,
}
