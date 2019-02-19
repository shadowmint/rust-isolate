use crate::IsolateIdentity;
use crate::isolate_runtime::IsolateRef;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use crate::Isolate;
use crate::IsolateChannel;
use std::thread;

pub struct IsolateRuntimeShared<T: Send + 'static> {
    pub refs: HashMap<IsolateIdentity, IsolateRef<T>>,
    isolate: Box<Isolate<T> + Send + 'static>,
}

impl<T: Send + 'static> IsolateRuntimeShared<T> {
    pub fn new(isolate: impl Isolate<T> + Send + 'static) -> Arc<Mutex<IsolateRuntimeShared<T>>> {
        Arc::new(Mutex::new(IsolateRuntimeShared {
            isolate: Box::new(isolate),
            refs: HashMap::new(),
        }))
    }

    /// Spawn a new isolate worker thread and run it
    pub fn spawn(&mut self) -> IsolateChannel<T> {
        let (ref_channel, worker_channel) = IsolateChannel::<T>::new();

        // Handle worker
        let worker_identity = IsolateIdentity::new();
        let mut worker = self.isolate.spawn(worker_identity.clone(), worker_channel);
        let handle = thread::spawn(move || {
            (worker)();
        });

        // Keep reference
        let consumer_channel = ref_channel.clone().unwrap();
        self.refs.insert(worker_identity, IsolateRef { channel: ref_channel, handle });

        return consumer_channel;
    }
}
