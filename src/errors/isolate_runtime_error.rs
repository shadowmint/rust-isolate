use std::error::Error;
use std::fmt::Display;
use std::fmt;
use std::sync::PoisonError;
use std::sync::MutexGuard;
use std::collections::HashMap;
use std::any::Any;
use crossbeam::channel::Sender;
use futures::sync::oneshot;
use crate::errors::isolate_error::IsolateError;

#[derive(Debug)]
pub enum IsolateRuntimeError {
    ConnectionFailed,
    IsolateHalted,
    SyncError,
}

impl Error for IsolateRuntimeError {}

impl Display for IsolateRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<PoisonError<MutexGuard<'_, HashMap<String, Sender<(Box<dyn Any + Send + 'static>, oneshot::Sender<Result<Option<Box<Any + Send + 'static>>, IsolateError>>)>>>>> for IsolateRuntimeError {
    fn from(_: PoisonError<MutexGuard<'_, HashMap<String, Sender<(Box<dyn Any + Send + 'static>, oneshot::Sender<Result<Option<Box<Any + Send + 'static>>, IsolateError>>)>>>>) -> Self {
        return IsolateRuntimeError::SyncError;
    }
}