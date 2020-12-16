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
                let var = Variable::Path(vec![variable.clone()])
                    .resolve(&ctx)?;
                let value = arg.clone().into_scalar().unwrap().to_integer().unwrap();
                let varg = var.into_scalar().unwrap().to_integer().unwrap();
                let exported =
                    Object::from_iter(once((variable.clone().into(), Value::scalar(varg + value))));
                ctx.push_vars(exported.clone());
            }
            Operation::PushCsv(variable, path) => {
                let var = Variable::Path(vec![variable.clone()])
                    .resolve(&ctx)?;
                let exists = path.as_path().exists();
                let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
                match var {
                    Value::Array(values) => {
                        if !exists {
                            wtr.write_record(&vec![variable.as_str()]).unwrap();
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
                            wtr.write_record(&vec![variable.as_str()]).unwrap();
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
            }
            Operation::Console(template) => {
                let output = ctx.apply(template.as_str());
                info!("{}", output);
            }
        }
        Ok(())
    }
}
