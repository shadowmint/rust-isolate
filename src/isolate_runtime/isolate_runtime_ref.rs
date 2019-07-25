use crate::isolate_runtime::isolate_runtime_shared::IsolateRuntimeShared;
use crate::IsolateChannel;
use crate::IsolateIdentity;
use crate::IsolateRuntimeError;
use std::sync::Arc;
use std::sync::Mutex;

pub struct IsolateRuntimeRef<T: Send + 'static> {
    shared: Arc<Mutex<IsolateRuntimeShared<T>>>,
}

impl<T: Send + 'static> IsolateRuntimeRef<T> {
    pub fn new(shared: Arc<Mutex<IsolateRuntimeShared<T>>>) -> IsolateRuntimeRef<T> {
        IsolateRuntimeRef { shared }
    }

    pub fn find(&self, identity: &IsolateIdentity) -> Option<IsolateChannel<T>> {
        match self.shared.lock() {
            Ok(inner) => match inner.refs.get(identity) {
                Some(r) => Some(r.channel.clone()),
                None => None,
            },
            Err(_) => None,
        }
    }

    /// Spawn a new isolate worker thread and run it
    pub fn spawn(&mut self) -> Result<IsolateChannel<T>, IsolateRuntimeError> {
        match self.shared.lock() {
            Ok(mut inner) => Ok(inner.spawn()),
            Err(_) => Err(IsolateRuntimeError::InternalSyncError),
        }
    }
}
