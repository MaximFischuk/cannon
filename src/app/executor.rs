use crate::configuration::manifest::CaptureEntry;
use core::slice::Iter;
use crate::app::context::Context;
use crate::app::hooks::Executable;
use crate::app::hooks::ExecutionResult;
use bytes::Bytes;
use std::time::Duration;
use hyper::StatusCode;

pub(crate) struct JobGroup<T> {
    name: String,
    jobs: Vec<(T, RunInfo)>,
}

pub(crate) struct RunInfo {
    pub repeats: u64,
    pub delay: Duration,
    pub captures: Vec<CaptureEntry>
}

impl RunInfo {

    pub fn new(repeats: u64, delay: Duration, captures: Vec<CaptureEntry>) -> Self {
        Self {
            repeats,
            delay,
            captures
        }
    }

}

impl <T> JobGroup<T> {
    pub fn new(name: String, jobs: Vec<(T, RunInfo)>) -> Self {
        Self {
            name,
            jobs
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, (T, RunInfo)> {
        self.jobs.iter()
    }

    #[inline]
    pub fn amount(&self) -> usize {
        self.jobs.len()
    }
}

impl <T> Executable for JobGroup<T> {
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

pub trait JobExecutionHooks<T, R> {
    fn before(&self, context: &mut Context) -> Result<String, String>;
    fn after(&self, context: &mut Context) -> Result<String, String>;
    fn execute(
        &self,
        context: &mut Context,
        sender: &impl SendMessage<T, R>,
    ) -> Result<Bytes, String>;
}

pub enum ExecutionCode {
    Http { code: StatusCode },
}

pub struct Status {
    execution_time: Duration,
    execution_code: ExecutionCode,
}

pub trait GetUuid {
    fn get_uuid(&self) -> uuid::Uuid;
}

pub trait SendMessage<T, R> {
    fn send(&self, data: T) -> R;
}
