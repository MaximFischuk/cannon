use mime::Mime;
use serde_derive::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    name: String,
    #[serde(with = "crate::reporter::serialize::mime_type")]
    r#type: Mime,
    source: String,
}
