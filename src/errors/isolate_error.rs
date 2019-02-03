use std::error::Error;

#[derive(Debug)]
pub struct IsolateError {}

impl IsolateError {
    pub fn from(_: impl Error) -> IsolateError {
        IsolateError {}
    }
}