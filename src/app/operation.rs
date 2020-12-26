use super::{capture::Resolvable, context::Context, error::Error};
use crate::configuration::manifest::{Operation, Variable};
use kstring::KString;
use liquid::{model::Value, Object};
use std::{
    fs::OpenOptions,
    io::Write,
    iter::{once, FromIterator},
};

pub trait Performable {
    fn perform(&self, ctx: &mut Context) -> Result<(), Error>;
}

impl Performable for Operation {
    fn perform(&self, ctx: &mut Context) -> Result<(), Error> {
        match self {
            Operation::Add(variable, arg) => {
                let var = Variable::Path(vec![variable.to_owned()]).resolve(&ctx)?;
                let result = math::sum(var, arg.to_owned())?;
                let exported = Object::from_iter(once((variable.clone().into(), result)));
                ctx.push_vars(exported.clone());
            }
            Operation::PushCsv(variable, path) => {
                let var = Variable::Path(vec![variable.clone()]).resolve(&ctx)?;
                output::push_csv(variable.as_str(), var, path)?;
            }
            Operation::Console(template) => {
                output::console(ctx.apply(template.as_str()).as_str())?;
            }
        }
        Ok(())
    }
}

mod math {

    use super::*;

    pub fn sum(arg1: Value, arg2: Value) -> Result<Value, Error> {
        let value1 = unwrap_integer(&arg1)?;
        let value2 = unwrap_integer(&arg2)?;

        Ok(Value::scalar(value1 + value2))
    }

    pub fn sub(arg1: Value, arg2: Value) -> Result<Value, Error> {
        let value1 = unwrap_integer(&arg1)?;
        let value2 = unwrap_integer(&arg2)?;

        Ok(Value::scalar(value1 - value2))
    }

    pub fn mul(arg1: Value, arg2: Value) -> Result<Value, Error> {
        let value1 = unwrap_integer(&arg1)?;
        let value2 = unwrap_integer(&arg2)?;

        Ok(Value::scalar(value1 * value2))
    }

    pub fn div(arg1: Value, arg2: Value) -> Result<Value, Error> {
        let value1 = unwrap_integer(&arg1)?;
        let value2 = unwrap_integer(&arg2)?;

        Ok(Value::scalar(value1 / value2))
    }

    pub fn module(arg1: Value, arg2: Value) -> Result<Value, Error> {
        let value1 = unwrap_integer(&arg1)?;
        let value2 = unwrap_integer(&arg2)?;

        Ok(Value::scalar(value1 % value2))
    }

    pub fn pow(arg1: Value, arg2: Value) -> Result<Value, Error> {
        let value1 = unwrap_integer(&arg1)?;
        let value2 = unwrap_integer(&arg2)?;

        let arg1 = value1 as u32;
        let arg2 = value2 as u32;

        Ok(Value::scalar(arg1.pow(arg2)))
    }

    fn unwrap_integer(value: &Value) -> Result<f64, Error> {
        if let Some(scalar) = value.clone().into_scalar() {
            scalar
                .to_float()
                .ok_or(Error::Internal("Cannot convert value to f64".to_owned()))
        } else {
            Err(Error::IncorrectValueType(
                value.as_view().type_name().to_owned(),
            ))
        }
    }
}

mod output {

    use std::path::PathBuf;

    use super::*;

    pub fn push_csv(header: &str, value: Value, path: &PathBuf) -> Result<(), Error> {
        let exists = path.as_path().exists();
        let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
        match value {
            Value::Array(values) => {
                if !exists {
                    wtr.write_record(&vec![header]).unwrap();
                }
                values.iter().for_each(|v| {
                    wtr.write_record(&vec![v.clone().into_scalar().unwrap().into_string()])
                        .unwrap()
                });
            }
            Value::Object(values) => {
                if !exists {
                    let keys: Vec<&str> = values.keys().map(KString::as_str).collect();
                    wtr.write_record(&keys).unwrap();
                }
                let values = values.values();
                values.for_each(|v| {
                    wtr.write_record(&vec![v.clone().into_scalar().unwrap().into_string()])
                        .unwrap()
                });
            }
            Value::Scalar(value) => {
                if !exists {
                    wtr.write_record(&vec![header]).unwrap();
                }
                wtr.write_record(&vec![value.into_string()]).unwrap();
            }
            _ => {}
        }
        let data = wtr.into_inner().unwrap();
        let mut file = OpenOptions::new()
            .append(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        file.write(data.as_ref()).unwrap();
        file.flush().unwrap();
        Ok(())
    }

    pub fn console(text: &str) -> Result<(), Error> {
        info!("{}", text);
        Ok(())
    }
}
