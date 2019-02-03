use crate::Isolate;
use crate::IsolateRuntime;
use futures::future::Future;
use std::any::Any;
use crate::errors::isolate_error::IsolateError;
use crate::IsolateTools;

struct Worker1 {}

impl Isolate for Worker1 {
    fn handle(&self, _: Box<Any + Send + 'static>, _: &IsolateRuntime) -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        IsolateTools::none()
    }
}

#[test]
fn test_send_message_to_worker() {
    let mut runtime = IsolateRuntime::new();
    runtime.set("worker1", Worker1 {}).unwrap();

    let channel = runtime.connect("worker1").unwrap();

    let mut r = tokio::runtime::Runtime::new().unwrap();
    r.block_on(channel.send("Hello".to_string()).then(|r| {
        assert!(r.is_ok());
        Ok(()) as Result<(), ()>
    })).unwrap();
}
