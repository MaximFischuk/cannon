use crate::app::capture::Resolvable;
use crate::app::CaptureValue;
use crate::app::Context;
use crate::configuration::manifest::AssertFunction;

pub trait Assertable<T> {
    fn assert(&self, ctx: &Context, data: &T) -> bool;
}

impl Assertable<CaptureValue> for AssertFunction {
    fn assert(&self, ctx: &Context, data: &CaptureValue) -> bool {
        trace!("Assertation value: {:#?} to {:#?}", data, self);
        match self {
            AssertFunction::Equal(var) => match var.resolve(ctx) {
                Ok(expected) => {
                    trace!("Check equals of {:?} to {:?}", expected, data);
                    data == &expected
                }
                Err(e) => {
                    error!("{}", e);
                    false
                }
            },
            AssertFunction::NotEqual(var) => match var.resolve(ctx) {
                Ok(expected) => data != &expected,
                Err(e) => {
                    error!("{}", e);
                    false
                }
            },
            AssertFunction::Matches(var, regex) => match var.resolve(ctx) {
                Ok(CaptureValue::Scalar(expected)) => {
                    regex.is_match(expected.into_string().as_str())
                }
                Err(e) => {
                    error!("{}", e);
                    false
                }
                _ => false,
            },
        }
    }
}

#[cfg(test)]
mod test {

    use std::iter::{once, FromIterator};

    use liquid::Object;

    use super::*;
    use crate::{app::ContextPool, configuration::manifest::Variable};

    #[test]
    fn test_value_equals_to_expexted_value() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::Equal(Variable::Value(CaptureValue::Scalar(
            liquid::model::scalar::Scalar::new(42),
        )));
        let result = assert_function.assert(&ContextPool::new().default_context(), &value);

        assert!(result);
    }

    #[test]
    fn test_value_equals_to_variable() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::Equal(Variable::Path(vec!["expect".into()]));
        let result = assert_function.assert(
            &ContextPool::with_vars(vec![("expect".into(), value.clone())]).default_context(),
            &value,
        );

        assert!(result);
    }

    #[test]
    fn test_value_not_equals_to_expexted_value() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::NotEqual(Variable::Value(CaptureValue::Scalar(
            liquid::model::scalar::Scalar::new(43),
        )));
        let result = assert_function.assert(&ContextPool::new().default_context(), &value);

        assert!(result);
    }

    #[test]
    fn test_value_not_equals_to_variable() {
        let expected = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::NotEqual(Variable::Path(vec!["expect".into()]));
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(43));
        let result = assert_function.assert(
            &ContextPool::with_vars(vec![("expect".into(), expected)]).default_context(),
            &value,
        );

        assert!(result);
    }

    #[test]
    fn test_value_equals_to_nested_variable() {
        let expected = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let value = expected.clone();
        let object = Object::from_iter(once(("value".into(), value)));
        let assert_function =
            AssertFunction::Equal(Variable::Path(vec!["expect".into(), "value".into()]));
        let result = assert_function.assert(
            &ContextPool::with_vars(vec![("expect".into(), CaptureValue::from(object))])
                .default_context(),
            &expected,
        );

        assert!(result);
    }
}
