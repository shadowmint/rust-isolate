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
pub enum IsolateChannelError {
    SyncError,
}

impl Error for IsolateChannelError {}

impl Display for IsolateChannelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

