use crate::app::capture::Resolvable;
use crate::app::CaptureValue;
use crate::app::Context;
use crate::configuration::manifest::AssertFunction;

use super::error::Error;

pub trait Assertable {
    fn assert(&self, ctx: &Context) -> Result<bool, Error>;
}

impl Assertable for AssertFunction {
    fn assert(&self, ctx: &Context) -> Result<bool, Error> {
        match self {
            AssertFunction::Equal(first, second) => match (first.resolve(ctx), second.resolve(ctx))
            {
                (Ok(expected), Ok(to)) => {
                    trace!("Check equals of {:?} to {:?}", expected, to);
                    Ok(to == expected)
                }
                (Ok(_), Err(e)) => Err(Error::AssertationFailed(e.to_string())),
                (Err(e), Ok(_)) => Err(Error::AssertationFailed(e.to_string())),
                (Err(e1), Err(e2)) => Err(Error::AssertationFailed(format!(
                    "Errors: '{}' and '{}'",
                    e1, e2
                ))),
            },
            AssertFunction::NotEqual(first, second) => {
                match (first.resolve(ctx), second.resolve(ctx)) {
                    (Ok(expected), Ok(to)) => {
                        trace!("Check equals of {:?} to {:?}", expected, to);
                        Ok(to != expected)
                    }
                    (Ok(_), Err(e)) => Err(Error::AssertationFailed(e.to_string())),
                    (Err(e), Ok(_)) => Err(Error::AssertationFailed(e.to_string())),
                    (Err(e1), Err(e2)) => Err(Error::AssertationFailed(format!(
                        "Errors: '{}' and '{}'",
                        e1, e2
                    ))),
                }
            }
            AssertFunction::Matches(var, regex) => match var.resolve(ctx) {
                Ok(CaptureValue::Scalar(expected)) => {
                    Ok(regex.is_match(expected.into_string().as_str()))
                }
                Err(e) => Err(Error::AssertationFailed(e.to_string())),
                Ok(value) => Err(Error::AssertationFailed(format!(
                    "Unsupported value type '{}'",
                    value.as_view().type_name()
                ))),
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
        let assert_function = AssertFunction::Equal(
            Variable::Value(CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42))),
            Variable::Value(value),
        );
        let result = assert_function.assert(&ContextPool::new().default_context());

        assert!(result.unwrap());
    }

    #[test]
    fn test_value_equals_to_variable() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::Equal(
            Variable::Path(vec!["expect".into()]),
            Variable::Value(value.clone()),
        );
        let result = assert_function.assert(
            &ContextPool::with_vars(vec![("expect".into(), value.clone())]).default_context(),
        );

        assert!(result.unwrap());
    }

    #[test]
    fn test_value_not_equals_to_expexted_value() {
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let assert_function = AssertFunction::NotEqual(
            Variable::Value(CaptureValue::Scalar(liquid::model::scalar::Scalar::new(43))),
            Variable::Value(value),
        );
        let result = assert_function.assert(&ContextPool::new().default_context());

        assert!(result.unwrap());
    }

    #[test]
    fn test_value_not_equals_to_variable() {
        let expected = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let value = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(43));
        let assert_function = AssertFunction::NotEqual(
            Variable::Path(vec!["expect".into()]),
            Variable::Value(value),
        );
        let result = assert_function
            .assert(&ContextPool::with_vars(vec![("expect".into(), expected)]).default_context());

        assert!(result.unwrap());
    }

    #[test]
    fn test_value_equals_to_nested_variable() {
        let expected = CaptureValue::Scalar(liquid::model::scalar::Scalar::new(42));
        let value = expected.clone();
        let object = Object::from_iter(once(("value".into(), value)));
        let assert_function = AssertFunction::Equal(
            Variable::Path(vec!["expect".into(), "value".into()]),
            Variable::Value(expected),
        );
        let result = assert_function.assert(
            &ContextPool::with_vars(vec![("expect".into(), CaptureValue::from(object))])
                .default_context(),
        );

        assert!(result.unwrap());
    }
}
