use crossbeam::channel::Sender;
use std::any::Any;
use futures::sync::oneshot::channel;
use crate::errors::isolate_channel_error::IsolateChannelError;
use futures::Future;
use futures::future::Either;
use futures::sync::oneshot;
use crate::errors::isolate_error::IsolateError;

/// IsolateChannel dispatches messages to an isolate instance
pub struct IsolateChannel {
    sender: Sender<(Box<Any + Send + 'static>, oneshot::Sender<Result<Option<Box<Any + Send + 'static>>, IsolateError>>)>
}

impl IsolateChannel {
    pub fn from(sender: &Sender<(Box<Any + Send + 'static>, oneshot::Sender<Result<Option<Box<Any + Send + 'static>>, IsolateError>>)>) -> IsolateChannel {
        return IsolateChannel {
            sender: sender.clone()
        };
    }

    /// Send an arbitrary value to the worker and get back a future for the processed response
    pub fn send(&self, message: impl Any + Send + 'static) -> impl Future<Item=Result<Option<Box<Any + Send + 'static>>, IsolateError>, Error=IsolateChannelError> {
        let message_boxed = Box::new(message) as Box<Any + Send + 'static>;
        self.send_boxed(message_boxed)
    }

    /// Send an existing boxed value to the worker and get back a future for the processed response.
    pub fn send_boxed(&self, message: Box<Any + Send + 'static>) -> impl Future<Item=Result<Option<Box<Any + Send + 'static>>, IsolateError>, Error=IsolateChannelError> {
        let (sx, rx) = channel();
        match self.sender.send((message, sx)) {
            Ok(_) => Either::A(rx.map_err(|_| IsolateChannelError::SyncError)),
            Err(_) => Either::B(futures::failed(IsolateChannelError::SyncError))
        }
    }
}