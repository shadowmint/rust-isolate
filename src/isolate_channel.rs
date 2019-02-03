use crossbeam::channel::Sender;
use std::any::Any;
use futures::sync::oneshot::channel;
use crate::errors::isolate_channel_error::IsolateChannelError;
use futures::Future;
use futures::future::Either;
use futures::sync::oneshot;

/// IsolateChannel dispatches messages to an isolate instance
pub struct IsolateChannel {
    sender: Sender<(Box<Any + Send + 'static>, oneshot::Sender<Option<Box<Any + Send + 'static>>>)>
}

impl IsolateChannel {
    pub fn from(sender: &Sender<(Box<Any + Send + 'static>, oneshot::Sender<Option<Box<Any + Send + 'static>>>)>) -> IsolateChannel {
        return IsolateChannel {
            sender: sender.clone()
        };
    }

    /// Send an arbitrary value to the worker and get back a future for the processed response
    pub fn send(&self, message: impl Any + Send + 'static) -> impl Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateChannelError> {
        let message_boxed = Box::new(message) as Box<Any + Send + 'static>;
        self.send_boxed(message_boxed)
    }

    /// Send an existing boxed value to the worker and get back a future for the processed response.
    /// This is particularly useful for workers that forward messages.
    pub fn send_boxed(&self, message: Box<Any + Send + 'static>) -> impl Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateChannelError> {
        let (sx, rx) = channel();
        match self.sender.send((message, sx)) {
            Ok(_) => Either::A(rx.map_err(|c| {
                println!("Cancelled: {:?}", c);
                IsolateChannelError::SyncError
            })),
            Err(e) => {
                println!("{:?}", e);
                Either::B(futures::failed(IsolateChannelError::SyncError))
            }
        }
    }
}