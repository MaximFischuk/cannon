use serde_derive::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    name: String,
    url: String,
    r#type: LinkType,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum LinkType {
    Issue,
    Tms,
    Custom,
}
