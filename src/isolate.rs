use crate::IsolateChannel;
use crate::IsolateIdentity;

/// Isolate the isolate worker that is run in its own thread to process tasks.
pub trait Isolate<T: Send + 'static> {
    /// Spawn is invoked when a new connection is opened to the isolate.
    /// It should return a function that can be invoked in a remote thread.
    /// The spawn function should handle incoming events on the channel until it closes.
    fn spawn(&self, identity: IsolateIdentity, channel: IsolateChannel<T>) -> Box<FnMut() + Send + 'static>;
}