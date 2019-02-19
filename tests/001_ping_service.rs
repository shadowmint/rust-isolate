use rust_isolate::Isolate;
use rust_isolate::IsolateIdentity;
use rust_isolate::IsolateChannel;
use rust_isolate::IsolateRuntime;
use rust_isolate::IsolateRuntimeWait;
use std::thread;
use std::time::Duration;

struct PingService {}

impl Isolate<String> for PingService {
    fn spawn(&self, _: IsolateIdentity, channel: IsolateChannel<String>) -> Box<FnMut() + Send + 'static> {
        Box::new(move || {
            loop {
                match channel.receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(r) => {
                        if r == "" {
                            break;
                        }
                    }
                    Err(e) => {
                        if !e.is_timeout() {
                            break;
                        }
                    }
                }
                channel.sender.send("PING".to_string()).unwrap();
            }
        })
    }
}

#[test]
pub fn main() {
    let mut runtime = IsolateRuntime::new(PingService {});

    let c1 = runtime.spawn().unwrap();
    let c2 = runtime.spawn().unwrap();
    let c3 = runtime.spawn().unwrap();

    thread::spawn(move || { test_isolate(c1) });
    thread::spawn(move || { test_isolate(c2) });
    thread::spawn(move || { test_isolate(c3) });

    runtime.wait();
}

fn test_isolate(channel: IsolateChannel<String>) {
    for _ in 1..10 {
        let value_out = channel.receiver.recv().unwrap();
        assert_eq!("PING", value_out)
    }
    channel.sender.send("".to_string()).unwrap();
}
