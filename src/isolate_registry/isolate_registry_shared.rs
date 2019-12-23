use std::sync::Mutex;
use std::sync::Arc;
use crate::isolate_registry::isolate_registry_error::IsolateRegistryError;
use crate::IsolateRuntimeRef;

use crate::Isolate;
use std::collections::HashMap;
use crate::IsolateRuntime;
use std::any::Any;
use crate::isolate_runtime::isolate_runtime_wait::IsolateRuntimeWait;

pub struct IsolateRegistryShared {
    registry: HashMap<String, Box<dyn Any + Send + 'static>>
}

impl IsolateRegistryShared {
    pub fn new() -> Arc<Mutex<IsolateRegistryShared>> {
        return Arc::new(Mutex::new(IsolateRegistryShared {
            registry: HashMap::new()
        }));
    }

    /// Bind a reference identity to a runtime instance.
    /// If the name is already used, raise an error.
    pub fn bind<T: Send + 'static>(&mut self, identity: &str, isolate: impl Isolate<T> + Send + 'static) -> Result<IsolateRuntimeRef<T>, IsolateRegistryError> {
        // Check the identity isn't already in use.
        if self.registry.contains_key(identity) {
            return Err(IsolateRegistryError::IdentityAlreadyInUse);
        }

        // Create a new runtime for this isolate
        let runtime = IsolateRuntime::new(isolate);
        let runtime_ref = runtime.as_ref();

        // Attach to the registry
        self.registry.insert(identity.to_string(), Box::new(runtime) as Box<dyn Any + Send + 'static>);
        return Ok(runtime_ref);
    }

    /// Find a specific runtime by name and type.
    /// Even if the name matches, if the downcast type ref is wrong, it'll return an error.
    pub fn find<T: Send + 'static>(&self, identity: &str) -> Result<IsolateRuntimeRef<T>, IsolateRegistryError> {
        match self.registry.get(identity) {
            Some(runtime_any) => {
                match runtime_any.downcast_ref::<IsolateRuntime<T>>() {
                    Some(runtime) => Ok(runtime.as_ref()),
                    None => Err(IsolateRegistryError::InvalidRuntimeType)
                }
            }
            None => Err(IsolateRegistryError::NoMatchingIdentity)
        }
    }

    /// Wait for all runtimes to halt
    pub fn wait(&self) {
        self.registry.iter().for_each(|(_, i)| {
            match i.downcast_ref::<&dyn IsolateRuntimeWait>() {
                Some(handle) => { handle.wait(); }
                None => {}
            }
        })
    }
}