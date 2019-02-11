use rust_isolate::Isolate;
use rust_isolate::IsolateIdentity;
use rust_isolate::IsolateChannel;
use rust_isolate::IsolateRuntimeRef;
use rust_isolate::IsolateRuntime;
use std::thread;

struct EchoService {}

impl Isolate<String> for EchoService {
    fn spawn(&self, _: IsolateIdentity, channel: IsolateChannel<String>, _: IsolateRuntimeRef<String>) -> Box<Fn() + Send + 'static> {
        Box::new(move || {
            loop {
                match channel.receiver.recv() {
                    Ok(r) => {
                        if r == "" {
                            break;
                        }
                        channel.sender.send(r).unwrap();
                    }
                    Err(_) => break
                }
            }
        })
    }
}

#[test]
pub fn main() {
    let mut runtime = IsolateRuntime::new(EchoService {});

    let c1 = runtime.spawn().unwrap();
    let c2 = runtime.spawn().unwrap();
    let c3 = runtime.spawn().unwrap();

    thread::spawn(move || { test_isolate(c1) });
    thread::spawn(move || { test_isolate(c2) });
    thread::spawn(move || { test_isolate(c3) });

    runtime.wait();
}

fn test_isolate(channel: IsolateChannel<String>) {
    for i in 1..10 {
        let value_in = format!("value: {}", i);
        channel.sender.send(value_in.clone()).unwrap();
        let value_out = channel.receiver.recv().unwrap();
        assert_eq!(value_in, value_out)
    }
    channel.sender.send("".to_string()).unwrap();
}