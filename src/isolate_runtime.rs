pub(crate) mod isolate_identity;
pub(crate) mod isolate_runtime_error;
pub(crate) mod isolate_runtime_shared;
pub(crate) mod isolate_runtime_ref;

use crate::IsolateIdentity;
use crate::Isolate;
use crate::IsolateChannel;
use crate::IsolateRuntimeError;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::sync::Mutex;
use crate::isolate_runtime::isolate_runtime_shared::IsolateRuntimeShared;
use crate::IsolateRuntimeRef;
use std::mem;
use std::collections::HashMap;

pub struct IsolateRef<T: Send + 'static> {
    channel: IsolateChannel<T>,
    handle: JoinHandle<()>,
}

pub struct IsolateRuntime<T: Send + 'static> {
    isolate: Box<Isolate<T> + Send + 'static>,
    shared: Arc<Mutex<IsolateRuntimeShared<T>>>,
}


impl<T: Send + 'static> IsolateRuntime<T> {
    /// Create a new runner with a specific isolate instance
    pub fn new(isolate: impl Isolate<T> + Send + 'static) -> IsolateRuntime<T> {
        IsolateRuntime {
            isolate: Box::new(isolate),
            shared: IsolateRuntimeShared::<T>::new(),
        }
    }

    /// Spawn a new isolate worker thread and run it
    pub fn spawn(&mut self) -> Result<IsolateChannel<T>, IsolateRuntimeError> {
        match self.shared.lock() {
            Ok(mut inner) => {
                let (ref_channel, worker_channel) = IsolateChannel::<T>::new();

                // Handle worker
                let worker_identity = IsolateIdentity::new();
                let worker = self.isolate.spawn(worker_identity.clone(), worker_channel, self.as_ref());
                let handle = thread::spawn(move || {
                    (worker)();
                });

                // Keep reference
                let consumer_channel = ref_channel.clone().unwrap();
                inner.refs.insert(worker_identity, IsolateRef { channel: ref_channel, handle });

                return Ok(consumer_channel);
            }
            Err(_) => Err(IsolateRuntimeError::InternalSyncError)
        }
    }

    /// Return a reference instance
    pub fn as_ref(&self) -> IsolateRuntimeRef<T> {
        IsolateRuntimeRef::new(self.shared.clone())
    }

    /// Halt this runner and wait for all its workers to shutdown
    pub fn wait(self) {
        match self.shared.lock() {
            Ok(mut inner) => {
                let mut refs = HashMap::new();
                mem::swap(&mut refs, &mut inner.refs);
                refs.into_iter().for_each(|(_, r)| {
                    r.channel.close();
                    let _ = r.handle.join();
                });
            }
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IsolateRuntime;
    use crate::Isolate;
    use crate::IsolateChannel;
    use crate::IsolateIdentity;
    use crate::IsolateRuntimeRef;

    struct TestIsolate {}

    #[derive(Debug)]
    enum TestIsolateEvent {
        Echo,
        Who,
        Identity(IsolateIdentity),
        Peer(IsolateIdentity),
        PeerGot,
    }

    impl Isolate<TestIsolateEvent> for TestIsolate {
        fn spawn(&self, identity: IsolateIdentity, channel: IsolateChannel<TestIsolateEvent>, runtime: IsolateRuntimeRef<TestIsolateEvent>) -> Box<Fn() + Send + 'static> {
            Box::new(move || {
                loop {
                    match channel.receiver.recv() {
                        Ok(v) => {
                            match v {
                                TestIsolateEvent::Who => {
                                    channel.sender.send(TestIsolateEvent::Identity(identity)).unwrap();
                                }
                                TestIsolateEvent::Peer(id) => {
                                    let target = runtime.find(id).unwrap();
                                    target.sender.send(TestIsolateEvent::PeerGot).unwrap();
                                }
                                _ => {
                                    // Ignore send errors; the connections may be broken by the tests.
                                    let _ = channel.sender.send(v);
                                }
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            })
        }
    }

    #[test]
    pub fn test_create_runner() {
        let _ = IsolateRuntime::new(TestIsolate {});
    }

    #[test]
    pub fn test_spawn_worker() {
        let mut runner = IsolateRuntime::new(TestIsolate {});

        let channel = runner.spawn().unwrap();
        channel.sender.send(TestIsolateEvent::Echo).unwrap();
        let output = channel.receiver.recv().unwrap();

        match output {
            TestIsolateEvent::Echo => {}
            _ => unreachable!()
        }
    }

    #[test]
    pub fn test_halt_runner() {
        let mut runner = IsolateRuntime::new(TestIsolate {});

        // The runtime will remain active until all open channel connections close.
        {
            let channel = runner.spawn().unwrap();
            channel.sender.send(TestIsolateEvent::Echo).unwrap();
        }

        runner.wait();
    }

    #[test]
    pub fn test_send_many_messages() {
        let mut runner = IsolateRuntime::new(TestIsolate {});

        let channel = runner.spawn().unwrap();

        for _ in 1..20 {
            channel.sender.send(TestIsolateEvent::Echo).unwrap();
        }

        for _ in 1..20 {
            let response = channel.receiver.recv();
            let output = response.unwrap();
            match output {
                TestIsolateEvent::Echo => {}
                _ => unreachable!()
            }
        }
    }

    #[test]
    pub fn test_broadcast_to_instance() {
        let mut runner = IsolateRuntime::new(TestIsolate {});
        let channel1 = runner.spawn().unwrap();

        channel1.sender.send(TestIsolateEvent::Who).unwrap();
        let response = channel1.receiver.recv().unwrap();
        match response {
            TestIsolateEvent::Identity(id) => {
                let channel2 = runner.as_ref().find(id).unwrap();

                // Remember, single push = single pull, you can't read the same event on channel1.receiver
                // because it's already been consumed by the channel2.receiver.
                channel2.sender.send(TestIsolateEvent::Echo).unwrap();
                let output2 = channel2.receiver.recv().unwrap();
                match output2 {
                    TestIsolateEvent::Echo => {}
                    _ => unreachable!()
                };
            }
            _ => unreachable!()
        }
    }

    #[test]
    pub fn test_peer_to_peer() {
        let mut runner = IsolateRuntime::new(TestIsolate {});
        let channel1 = runner.spawn().unwrap();
        let channel2 = runner.spawn().unwrap();

        channel1.sender.send(TestIsolateEvent::Who).unwrap();
        let id = channel1.receiver.recv().unwrap();
        match id {
            TestIsolateEvent::Identity(i) => {
                channel2.sender.send(TestIsolateEvent::Peer(i)).unwrap();
                let output = channel1.receiver.recv().unwrap();
                match output {
                    TestIsolateEvent::PeerGot => {}
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        }
    }
}
