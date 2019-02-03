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

mod isolate_runtime_instance;

/// IsolateRuntime is the base container to managing and communicating with Isolate instances.
#[derive(Clone)]
pub struct IsolateRuntime {
    isolates: Arc<Mutex<HashMap<String, Sender<(Box<Any + Send + 'static>, oneshot::Sender<Option<Box<Any + Send + 'static>>>)>>>>
}


impl IsolateRuntime {
    /// Create a new empty instance
    pub fn new() -> IsolateRuntime {
        IsolateRuntime {
            isolates: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    /// Add a new isolate to this runtime
    pub fn set(&mut self, identity: &str, isolate: impl Isolate + Send + 'static) -> Result<(), IsolateChannelError> {
        let (sx, rx) = crossbeam::channel::unbounded();

        // Spawn a new instance
        let runtime = self.clone();
        thread::spawn(move || {
            IsolateRuntimeInstance::new(rx, isolate, runtime).run();
        });

        // Save the reference to the instance
        let mut isolate_ref = self.isolates.lock()?;
        isolate_ref.insert(identity.to_string(), sx);
        Ok(())
    }

    /// Open a channel to a named endpoint
    pub fn connect(&self, identity: &str) -> Result<IsolateChannel, IsolateChannelError> {
        let isolate_ref = self.isolates.lock()?;
        match isolate_ref.get(identity) {
            Some(channel) => Ok(IsolateChannel::from(channel)),
            None => Err(IsolateChannelError::ConnectionFailed)
        }
    }
}