use crate::errors::StatefulError;
use rust_isolate::Isolate;
use rust_isolate::IsolateChannel;
use rust_isolate::IsolateIdentity;
use rust_isolate::IsolateRuntime;
use rust_isolate::IsolateRuntimeWait;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

// Errors
mod errors {
    use std::error::Error;

    pub enum StatefulError {
        Halted,
        InnerError(String),
    }

    impl StatefulError {
        pub fn halted() -> StatefulError {
            StatefulError::Halted
        }

        pub fn from(err: impl Error) -> StatefulError {
            StatefulError::InnerError(err.description().to_string())
        }
    }
}

// Events
enum StatefulEvent {
    Add(u32),
    Sub(u32),
    Halt,
}

// Global shared state
struct SharedState {
    global_total: i32,
}

// State for each individual worker
#[derive(Clone)]
struct StatefulService {
    shared: Arc<Mutex<SharedState>>,
    total: i32,
}

impl StatefulService {
    pub fn new(shared: Arc<Mutex<SharedState>>) -> StatefulService {
        StatefulService { shared, total: 0 }
    }

    pub fn dispatch(&mut self, event: StatefulEvent) -> Result<(), StatefulError> {
        match event {
            StatefulEvent::Add(n) => {
                self.total += n as i32;
                self.shared.lock().unwrap().global_total += n as i32;
                Ok(())
            }
            StatefulEvent::Sub(n) => {
                self.total -= n as i32;
                self.shared.lock().unwrap().global_total -= n as i32;
                Ok(())
            }
            StatefulEvent::Halt => Err(StatefulError::halted()),
        }
    }

    pub fn event_loop(&mut self, channel: &IsolateChannel<StatefulEvent>) {
        loop {
            let result = match channel.receiver.recv() {
                Ok(event) => self.dispatch(event),
                Err(err) => Err(StatefulError::from(err)),
            };
            if result.is_err() {
                break;
            }
        }
    }
}

impl Isolate<StatefulEvent> for StatefulService {
    fn spawn(
        &self,
        _: IsolateIdentity,
        channel: IsolateChannel<StatefulEvent>,
    ) -> Box<FnMut() + Send + 'static> {
        let mut instance = self.clone();
        Box::new(move || {
            instance.event_loop(&channel);
        })
    }
}

#[test]
pub fn main() {
    let shared = Arc::new(Mutex::new(SharedState { global_total: 0 }));
    let mut runtime = IsolateRuntime::new(StatefulService::new(shared.clone()));

    let c1 = runtime.spawn().unwrap();
    let c2 = runtime.spawn().unwrap();
    let c3 = runtime.spawn().unwrap();

    c1.sender.send(StatefulEvent::Add(10)).unwrap();
    c1.sender.send(StatefulEvent::Add(10)).unwrap();
    c2.sender.send(StatefulEvent::Add(1)).unwrap();
    c3.sender.send(StatefulEvent::Sub(100)).unwrap();

    thread::sleep(Duration::from_millis(100));

    c1.sender.send(StatefulEvent::Halt).unwrap();
    c2.sender.send(StatefulEvent::Halt).unwrap();
    c3.sender.send(StatefulEvent::Halt).unwrap();

    runtime.wait();

    assert_eq!(shared.lock().unwrap().global_total, -79);
}
