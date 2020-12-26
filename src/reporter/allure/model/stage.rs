use serde_derive::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Scheduled,
    Running,
    Finished,
    Pending,
    Interrupted,
}
