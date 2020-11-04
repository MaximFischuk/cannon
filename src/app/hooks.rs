use crate::app::context::Context;

pub type ExecutionResult = Result<String, String>;

pub(crate) trait Executable {
    fn before_all(&self, ctx: &mut Context) -> ExecutionResult;
    fn before_each(&self, ctx: &mut Context) -> ExecutionResult;
    fn after_all(&self, ctx: &mut Context) -> ExecutionResult;
    fn after_each(&self, ctx: &mut Context) -> ExecutionResult;
    fn execute(&self, ctx: &mut Context) -> ExecutionResult;
}
