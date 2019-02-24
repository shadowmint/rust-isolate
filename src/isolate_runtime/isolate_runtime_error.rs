use std::fmt::Display;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum IsolateRuntimeError {
    InternalSyncError,
    InvalidIdentity(String)
}

impl Error for IsolateRuntimeError {}

impl Display for IsolateRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}