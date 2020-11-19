use crate::app::capture::Resolvable;
use crate::app::CaptureValue;
use crate::app::Context;
use crate::configuration::manifest::AssertFunction;
use crate::configuration::manifest::Functor;

pub trait Assertable<T> {
    fn assert(&self, ctx: &Context, data: &T) -> bool;
}

impl Assertable<CaptureValue> for Functor {
    fn assert(&self, ctx: &Context, data: &CaptureValue) -> bool {
        match self {
            Functor::Assert {
                function,
                message: _,
            } => assert_value(ctx, data, &function),
            Functor::Matches(_pattern) => {
                unimplemented!();
            }
        }
    }
}

fn assert_value(ctx: &Context, value: &CaptureValue, assert: &AssertFunction) -> bool {
    trace!("Assertation value: {:#?} to {:#?}", value, assert);
    match assert {
        AssertFunction::Equal(var) => match var.resolve(ctx) {
            Ok(expected) => {
                trace!("Check equals of {:?} to {:?}", expected, value);
                value == &expected
            }
            Err(e) => {
                error!("{}", e);
                false
            }
        },
        AssertFunction::NotEqual(var) => match var.resolve(ctx) {
            Ok(expected) => value != &expected,
            Err(e) => {
                error!("{}", e);
                false
            }
        },
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::configuration::manifest::AssertParamValueVar;

    #[test]
    fn test_value_equals_to_expexted_value() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::Equal(AssertParamValueVar::Value(
            CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42)),
        ));
        let result = assert_value(&Context::new(), &value, &assert_function);

        assert!(result);
    }

    #[test]
    fn test_value_equals_to_variable() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::Equal(AssertParamValueVar::Var("expect".into()));
        let result = assert_value(
            &Context::with_vars(vec![("expect".into(), value.clone())]),
            &value,
            &assert_function,
        );

        assert!(result);
    }

    #[test]
    fn test_value_not_equals_to_expexted_value() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::NotEqual(AssertParamValueVar::Value(
            CaptureValue::Scalar(liquid::model::scalar::Scalar::new(43)),
        ));
        let result = assert_value(&Context::new(), &value, &assert_function);

        assert!(result);
    }

    #[test]
    fn test_value_not_equals_to_variable() {
        let expected = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::NotEqual(AssertParamValueVar::Var("expect".into()));
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(43));
        let result = assert_value(
            &Context::with_vars(vec![("expect".into(), expected.clone())]),
            &value,
            &assert_function,
        );

        assert!(result);
    }
}
