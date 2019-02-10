use crate::Isolate;
use std::any::Any;
use crate::IsolateIdentity;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use crate::IsolateChannel;

pub(crate) mod isolate_identity;
pub(crate) mod isolate_runtime_error;
mod isolate_runner;

#[derive(Clone)]
pub struct IsolateRuntime {
    inner: Arc<Mutex<IsolateRuntimeInner>>
}

pub struct IsolateRuntimeInner {
    runners: HashMap<IsolateIdentity, Box<Any>>
}

impl IsolateRuntime {
    pub fn new() -> IsolateRuntime {
        IsolateRuntime {
            inner: Arc::new(Mutex::new(IsolateRuntimeInner {
                runners: HashMap::new()
            }))
        }
    }

    pub fn set<T>(&self, isolate_identity: &str, isolate: impl Isolate<T> + Send + 'static) -> Result<(), ()> {}

    pub fn halt(&self, isolate_identity: &str) -> Result<(), ()> {}

    pub fn find<T>(&self, isolate_identity: &str, identity: IsolateIdentity) -> Result<IsolateChannel<T>, ()> {}
}