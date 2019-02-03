use std::error::Error;
use std::fmt::Display;
use std::fmt;
use std::sync::PoisonError;
use std::sync::MutexGuard;
use std::collections::HashMap;
use std::any::Any;
use crossbeam::channel::Sender;
use futures::sync::oneshot;

#[derive(Debug)]
pub enum IsolateChannelError {
    ConnectionFailed,
    SyncError,
}

impl Error for IsolateChannelError {}

impl Display for IsolateChannelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<PoisonError<MutexGuard<'_, HashMap<String, Sender<(Box<dyn Any + Send + 'static>, oneshot::Sender<Option<Box<Any + Send + 'static>>>)>>>>> for IsolateChannelError {
    fn from(_: PoisonError<MutexGuard<'_, HashMap<String, Sender<(Box<dyn Any + Send + 'static>, oneshot::Sender<Option<Box<Any + Send + 'static>>>)>>>>) -> Self {
        return IsolateChannelError::SyncError;
    }
}