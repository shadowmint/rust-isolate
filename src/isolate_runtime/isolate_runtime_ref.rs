use std::sync::Arc;
use std::sync::Mutex;
use crate::isolate_runtime::isolate_runtime_shared::IsolateRuntimeShared;

pub struct IsolateRuntimeRef<T: Send + 'static> {
    shared: Arc<Mutex<IsolateRuntimeShared<T>>>,
}

impl<T: Send + 'static> IsolateRuntimeRef<T> {
    pub fn new(shared: Arc<Mutex<IsolateRuntimeShared<T>>>) -> IsolateRuntimeRef<T> {
        IsolateRuntimeRef {
            shared
        }
    }
}
