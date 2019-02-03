use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum IsolateError {
    Failed,
    Reason(String),
}

impl IsolateError {
    pub fn from_error(_: impl Error) -> IsolateError {
        IsolateError::Failed
    }
}

impl From<&str> for IsolateError {
    fn from(s: &str) -> Self {
        IsolateError::Reason(s.to_string())
    }
}

impl Error for IsolateError {}

impl fmt::Display for IsolateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}