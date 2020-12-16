use crate::app::error::Error;
use crate::app::hooks::Executable;
use crate::app::hooks::ExecutionResult;
use crate::configuration::manifest::CaptureEntry;
use crate::{app::capture::CaptureValue, configuration::manifest::Operation};
use crate::{app::context::Context, configuration::manifest::Assertion};
use bytes::Bytes;
use core::slice::Iter;
use std::{collections::HashMap, time::Duration};

pub(crate) struct JobGroup<T> {
    name: String,
    jobs: Vec<(T, RunInfo)>,
}

pub(crate) struct RunInfo {
    pub name: String,
    pub id: uuid::Uuid,
    pub repeats: u64,
    pub delay: Duration,
    pub captures: Vec<CaptureEntry>,
    pub operations: Vec<Operation>,
    pub assertions: Vec<Assertion>,
}

#[derive(Builder)]
pub struct ExecutionResponse {
    body: Bytes,
    additional: HashMap<String, CaptureValue>,
    execution_time: Duration,
}

impl RunInfo {
    pub fn new(
        name: String,
        repeats: u64,
        delay: Duration,
        captures: Vec<CaptureEntry>,
        operations: Vec<Operation>,
        assertions: Vec<Assertion>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            repeats,
            delay,
            captures,
            operations,
            assertions,
        }
    }
}

impl<T> JobGroup<T> {
    pub fn new(name: String, jobs: Vec<(T, RunInfo)>) -> Self {
        Self { name, jobs }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, (T, RunInfo)> {
        self.jobs.iter()
    }

    #[inline]
    pub fn amount(&self) -> usize {
        self.jobs.len()
    }

    #[inline]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl<T> Executable for JobGroup<T> {
    fn before_all(&self, _context: &mut Context) -> ExecutionResult {
        todo!()
    }

    fn before_each(&self, _context: &mut Context) -> ExecutionResult {
        todo!()
    }

    fn after_all(&self, _context: &mut Context) -> ExecutionResult {
        todo!()
    }

    fn after_each(&self, _context: &mut Context) -> ExecutionResult {
        todo!()
    }

    fn execute(&self, _context: &mut Context) -> ExecutionResult {
        todo!()
    }
}

impl ExecutionResponse {
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    pub fn additional(&self) -> &HashMap<String, CaptureValue> {
        &self.additional
    }

    pub fn execution_time(&self) -> Duration {
        self.execution_time
    }

    pub fn builder() -> ExecutionResponseBuilder {
        ExecutionResponseBuilder::default()
    }
}

// impl From<Bytes> for ExecutionResponse {
//     fn from(body: Bytes) -> Self {
//         Self::new(body, CaptureValue::Nil, Duration::default())
//     }
// }

// impl From<CaptureValue> for ExecutionResponse {
//     fn from(additional: CaptureValue) -> Self {
//         Self::new(Bytes::default(), additional, Duration::default())
//     }
// }

pub trait JobExecutionHooks {
    fn before(&self, context: &Context) -> Result<String, String>;
    fn after(&self, context: &Context) -> Result<String, String>;
    fn execute(&self, context: &Context) -> Result<ExecutionResponse, Error>;
}
