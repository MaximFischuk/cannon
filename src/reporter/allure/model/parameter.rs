use serde_derive::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    name: String,
    value: String,
    hidden: bool,
    excluded: bool,
}
