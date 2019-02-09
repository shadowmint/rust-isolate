use crate::IsolateIdentity;
use std::any::Any;
use crate::Isolate;
use crate::IsolateChannel;
use crate::IsolateRuntimeError;
use std::thread;
use std::thread::JoinHandle;
use std::collections::HashMap;

pub struct IsolateRef<T: Send + 'static> {
    channel: IsolateChannel<T>,
    handle: JoinHandle<()>,
}

pub struct IsolateRunner<T: Send + 'static> {
    identity: IsolateIdentity,
    isolate: Box<Isolate<T> + 'static>,
    refs: HashMap<IsolateIdentity, IsolateRef<T>>,
}

impl<T: Clone + Send + 'static> IsolateRunner<T> {
    /// Create a new runner with a specific isolate instance
    pub fn new(isolate: impl Isolate<T> + 'static) -> IsolateRunner<T> {
        IsolateRunner {
            identity: IsolateIdentity::new(),
            isolate: Box::new(isolate),
            refs: HashMap::new(),
        }
    }

    /// Spawn a new isolate worker thread and run it
    pub fn spawn(&mut self) -> Result<IsolateChannel<T>, IsolateRuntimeError> {
        let (ref_channel, worker_channel) = IsolateChannel::<T>::new();

        // Handle worker
        let worker_identity = IsolateIdentity::new();
        let worker = self.isolate.spawn(worker_identity, worker_channel);
        let handle = thread::spawn(move || {
            (worker)();
        });

        // Keep reference
        let consumer_channel = ref_channel.clone().unwrap();
        self.refs.insert(worker_identity, IsolateRef { channel: ref_channel, handle });

        return Ok(consumer_channel);
    }

    /// Publish a message to a specific worker
    pub fn publish(&self, target: Option<IsolateIdentity>, message: T) {}

    /// Publish a message to all workers
    pub fn broadcast(&self, message: T) {
        self.refs.iter().for_each(|(_, r)| {
            r.channel.sender.send(message.clone());
        });
    }

    /// Halt this runner and wait for all its workers to shutdown
    pub fn halt(self) {
        self.refs.into_iter().for_each(|(_, r)| {
            r.channel.close();
            r.handle.join();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::IsolateRunner;
    use crate::Isolate;
    use crate::IsolateChannel;
    use crate::IsolateIdentity;

    struct TestIsolate {}

    #[derive(Debug, Clone)]
    enum TestIsolateEvent {
        Open,
        Who,
        Identity(IsolateIdentity),
        Peer(IsolateIdentity),
        PeerGot,
        Close,
    }

    impl Isolate<TestIsolateEvent> for TestIsolate {
        fn spawn(&self, identity: IsolateIdentity, channel: IsolateChannel<TestIsolateEvent>) -> Box<Fn() + Send + 'static> {
            Box::new(move || {
                match channel.receiver.recv() {
                    Ok(v) => {
                        println!("Got: {:?}", v);
                        match v {
                            TestIsolateEvent::Who => {
                                println!("Identity resp?");
                                channel.sender.send(TestIsolateEvent::Identity(identity));
                            }
                            TestIsolateEvent::Peer(id) => {}
                            _ => {
                                println!("default resp");
                                channel.sender.send(v);
                            }
                        }
                    }
                    Err(_) => {}
                }
            })
        }
    }

    #[test]
    pub fn test_create_runner() {
        let _ = IsolateRunner::new(TestIsolate {});
    }

    #[test]
    pub fn test_spawn_worker() {
        let mut runner = IsolateRunner::new(TestIsolate {});

        let channel = runner.spawn().unwrap();
        channel.sender.send(TestIsolateEvent::Open).unwrap();
        let output = channel.receiver.recv().unwrap();

        match output {
            TestIsolateEvent::Open => {}
            _ => unreachable!()
        }
    }

    #[test]
    pub fn test_halt_runner() {
        let mut runner = IsolateRunner::new(TestIsolate {});

        let channel = runner.spawn().unwrap();
        channel.sender.send(TestIsolateEvent::Open).unwrap();

        runner.halt();
    }

    #[test]
    pub fn test_broadcast() {
        let mut runner = IsolateRunner::new(TestIsolate {});
        let channel1 = runner.spawn().unwrap();
        let channel2 = runner.spawn().unwrap();

        runner.broadcast(TestIsolateEvent::Open);

        let output = channel1.receiver.recv().unwrap();
        match output {
            TestIsolateEvent::Open => {}
            _ => unreachable!()
        }

        let output = channel2.receiver.recv().unwrap();
        match output {
            TestIsolateEvent::Open => {}
            _ => unreachable!()
        }
    }

    #[test]
    pub fn test_peer_to_peer() {
        let mut runner = IsolateRunner::new(TestIsolate {});
        let channel1 = runner.spawn().unwrap();
        let channel2 = runner.spawn().unwrap();

        channel1.sender.send(TestIsolateEvent::Who).unwrap();
        let id = channel1.receiver.recv().unwrap();
        println!("Resp: {:?}", id);
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