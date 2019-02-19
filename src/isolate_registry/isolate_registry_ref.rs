use std::sync::Arc;
use std::sync::Mutex;
use crate::isolate_registry::isolate_registry_shared::IsolateRegistryShared;
use crate::isolate_registry::isolate_registry_error::IsolateRegistryError;
use crate::IsolateRuntimeRef;

#[derive(Clone)]
pub struct IsolateRegistryRef {
    shared: Arc<Mutex<IsolateRegistryShared>>,
}

impl IsolateRegistryRef {
    pub fn new(shared: Arc<Mutex<IsolateRegistryShared>>) -> IsolateRegistryRef {
        IsolateRegistryRef {
            shared
        }
    }

    /// Find a runtime by name, from the shared registry
    pub fn find<T: Send + 'static>(&self, identity: &str) -> Result<IsolateRuntimeRef<T>, IsolateRegistryError> {
        match self.shared.lock() {
            Ok(shared) => shared.find(identity),
            Err(_) => Err(IsolateRegistryError::InternalSyncError)
        }
    }
}
