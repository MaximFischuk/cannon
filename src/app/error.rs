use std::fmt;
use std::iter::repeat;

/// An error that occurred during parsing or compiling a regular expression.
#[derive(Clone, PartialEq)]
#[non_exhaustive] 
pub enum Error {
    AssertationFailed(String),
    ValueNotFound(String),
    Connection(String),
    Syntax(String),
    Internal(String),
}

impl std::error::Error for Error {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match *self {
            Error::AssertationFailed(ref err) => err,
            Error::ValueNotFound(ref err) => err,
            Error::Connection(ref err) => err,
            Error::Syntax(ref err) => err,
            Error::Internal(ref err) => err,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::AssertationFailed(ref err) => err.fmt(f),
            Error::ValueNotFound(ref err) => err.fmt(f),
            Error::Connection(ref err) => err.fmt(f),
            Error::Syntax(ref err) => err.fmt(f),
            Error::Internal(ref err) => err.fmt(f),
        }
    }
}

// We implement our own Debug implementation so that we show nicer syntax
// errors when people use `Regex::new(...).unwrap()`. It's a little weird,
// but the `Syntax` variant is already storing a `String` anyway, so we might
// as well format it nicely.
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::AssertationFailed(ref err) => {
                let hr: String = repeat('~').take(79).collect();
                writeln!(f, "Syntax(")?;
                writeln!(f, "{}", hr)?;
                writeln!(f, "{}", err)?;
                writeln!(f, "{}", hr)?;
                write!(f, ")")?;
                Ok(())
            }
            Error::ValueNotFound(ref err) => {
                let hr: String = repeat('~').take(79).collect();
                writeln!(f, "Syntax(")?;
                writeln!(f, "{}", hr)?;
                writeln!(f, "{}", err)?;
                writeln!(f, "{}", hr)?;
                write!(f, ")")?;
                Ok(())
            }
            Error::Connection(ref err) => {
                let hr: String = repeat('~').take(79).collect();
                writeln!(f, "Syntax(")?;
                writeln!(f, "{}", hr)?;
                writeln!(f, "{}", err)?;
                writeln!(f, "{}", hr)?;
                write!(f, ")")?;
                Ok(())
            }
            Error::Syntax(ref err) => {
                let hr: String = repeat('~').take(79).collect();
                writeln!(f, "Syntax(")?;
                writeln!(f, "{}", hr)?;
                writeln!(f, "{}", err)?;
                writeln!(f, "{}", hr)?;
                write!(f, ")")?;
                Ok(())
            }
            Error::Internal(ref err) => {
                let hr: String = repeat('~').take(79).collect();
                writeln!(f, "Syntax(")?;
                writeln!(f, "{}", hr)?;
                writeln!(f, "{}", err)?;
                writeln!(f, "{}", hr)?;
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}
