pub(crate) mod isolate_registry_shared;
pub(crate) mod isolate_registry_error;
pub(crate) mod isolate_registry_ref;

use self::isolate_registry_shared::IsolateRegistryShared;
use std::sync::Mutex;
use std::sync::Arc;
use crate::isolate_registry::isolate_registry_ref::IsolateRegistryRef;
use crate::isolate_registry::isolate_registry_error::IsolateRegistryError;
use crate::Isolate;
use crate::IsolateRuntimeRef;

/// IsolateRegistry allows you to bind external constant id strings to runtime instances.
/// Effectively this is a lookup cache to find the appropriate IsolateRuntime that can
/// be shared amongst isolate instances.
pub struct IsolateRegistry {
    shared: Arc<Mutex<IsolateRegistryShared>>
}

impl IsolateRegistry {
    pub fn new() -> IsolateRegistry {
        IsolateRegistry {
            shared: IsolateRegistryShared::new()
        }
    }

    /// Return a reference instance
    pub fn as_ref(&self) -> IsolateRegistryRef {
        IsolateRegistryRef::new(self.shared.clone())
    }

    /// Bind a new runtime instance to this registry with a specific name
    pub fn bind<T: Send + 'static>(&mut self, identity: &str, isolate: impl Isolate<T> + Send + 'static) -> Result<IsolateRuntimeRef<T>, IsolateRegistryError> {
        match self.shared.lock() {
            Ok(mut shared) => shared.bind(identity, isolate),
            Err(_) => Err(IsolateRegistryError::InternalSyncError)
        }
    }

    /// Find a specific runtime by name and type.
    /// Even if the name matches, if the downcast type ref is wrong, it'll return an error.
    pub fn find<T: Send + 'static>(&self, identity: &str) -> Result<IsolateRuntimeRef<T>, IsolateRegistryError> {
        match self.shared.lock() {
            Ok(shared) => shared.find(identity),
            Err(_) => Err(IsolateRegistryError::InternalSyncError)
        }
    }

    /// Wait for all runtimes to halt
    pub fn wait(self) {
        match self.shared.lock() {
            Ok(shared) => shared.wait(),
            Err(_) => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::IsolateRegistry;

    #[test]
    pub fn test_create_registry() {
        let _ = IsolateRegistry::new();
    }
}