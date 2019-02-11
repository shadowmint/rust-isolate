use crate::IsolateIdentity;
use crate::isolate_runtime::IsolateRef;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub struct IsolateRuntimeShared<T: Send + 'static> {
    pub refs: HashMap<IsolateIdentity, IsolateRef<T>>,
}

impl<T: Send + 'static> IsolateRuntimeShared<T> {
    pub fn new() -> Arc<Mutex<IsolateRuntimeShared<T>>> {
        Arc::new(Mutex::new(IsolateRuntimeShared {
            refs: HashMap::new(),
        }))
    }
}
