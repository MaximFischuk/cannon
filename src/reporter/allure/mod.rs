use std::time::Duration;

pub mod model;
pub mod test;

#[derive(Debug, Builder)]
pub struct Suite {
    id: uuid::Uuid,
    name: String,
    groups: Vec<Group>,
    start: Duration,
    stop: Duration,
}

#[derive(Debug, Builder, Clone)]
pub struct Group {
    id: uuid::Uuid,
    name: String,
    tests: Vec<Test>,
    start: Duration,
    stop: Duration,
}

#[derive(Debug, Builder, Clone)]
pub struct Test {
    id: uuid::Uuid,
    name: String,
    start: Duration,
    stop: Duration,
    status: Status,
}

#[derive(Debug, Clone)]
pub enum Status {
    Skipped,
    Failed,
    Passed,
    Broken,
}

pub trait Lifecycle {}

pub trait Adapter {}
