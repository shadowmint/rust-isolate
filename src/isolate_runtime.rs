use crate::errors::isolate_channel_error::IsolateChannelError;
use crate::isolate_channel::IsolateChannel;
use crate::isolate::Isolate;
use crossbeam;
use crossbeam::channel::Sender;
use std::thread;
use crate::isolate_runtime::isolate_runtime_instance::IsolateRuntimeInstance;
use std::sync::Arc;
use std::sync::Mutex;
use std::any::Any;
use std::collections::HashMap;
use futures::sync::oneshot;
use crate::errors::isolate_error::IsolateError;
use crate::errors::isolate_runtime_error::IsolateRuntimeError;

mod isolate_runtime_instance;

/// IsolateRuntime is the base container to managing and communicating with Isolate instances.
#[derive(Clone)]
pub struct IsolateRuntime {
    isolates: Arc<Mutex<HashMap<String, Sender<(Box<Any + Send + 'static>, oneshot::Sender<Result<Option<Box<Any + Send + 'static>>, IsolateError>>)>>>>
}


impl IsolateRuntime {
    /// Create a new empty instance
    pub fn new() -> IsolateRuntime {
        IsolateRuntime {
            isolates: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    /// Add a new isolate to this runtime
    pub fn set(&self, identity: &str, isolate: impl Isolate + Send + 'static) -> Result<(), IsolateRuntimeError> {
        let (sx, rx) = crossbeam::channel::unbounded();

        // Spawn a new instance
        let runtime = self.clone();
        let isolate = IsolateRuntimeInstance::new(identity, rx, isolate, runtime);
        thread::spawn(move || isolate.run());

        // Save the reference to the instance
        let mut isolate_ref = self.isolates.lock()?;
        isolate_ref.insert(identity.to_string(), sx);
        Ok(())
    }

    /// Remove an isolate.
    /// The isolate will remain running until the current event is processed.
    pub fn halt(&self, identity: &str) -> Result<(), IsolateRuntimeError> {
        let mut isolate_ref = self.isolates.lock()?;
        isolate_ref.remove(identity);
        Ok(())
    }

    /// Open a channel to a named endpoint
    pub fn connect(&self, identity: &str) -> Result<IsolateChannel, IsolateRuntimeError> {
        let isolate_ref = self.isolates.lock()?;
        match isolate_ref.get(identity) {
            Some(channel) => Ok(IsolateChannel::from(channel)),
            None => Err(IsolateRuntimeError::ConnectionFailed)
        }
    }
}