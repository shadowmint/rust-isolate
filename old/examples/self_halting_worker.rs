use crate::Isolate;
use crate::IsolateRuntime;
use futures::future::Future;
use std::any::Any;
use crate::errors::isolate_error::IsolateError;
use crate::IsolateTools;
use crate::isolate::IsolateContext;

struct Worker1 {}

impl Isolate for Worker1 {
    fn handle(&self, input: Box<Any + Send + 'static>, mut context: IsolateContext) -> Box<Future<Item=Result<Option<Box<Any + Send + 'static>>, IsolateError>, Error=IsolateError> + Send + 'static> {
        let output = match input.downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => "No input".to_string()
        };
        if output == "HALT" {
            context.runtime.halt(context.identity).unwrap();
        }
        IsolateTools::some_as_future(output)
    }
}

#[test]
fn test_send_message_to_worker() {
    let mut r = tokio::runtime::Runtime::new().unwrap();
    let mut runtime = IsolateRuntime::new();

    runtime.set("worker1", Worker1 {}).unwrap();

    // Send on message
    let channel = runtime.connect("worker1").unwrap();
    r.block_on(channel.send("World".to_string()).then(|r| {
        assert!(r.is_ok());
        assert_eq!(r.unwrap().unwrap().unwrap().downcast::<String>().unwrap().as_ref(), "World");
        Ok(()) as Result<(), ()>
    })).unwrap();

    // Halt the worker
    r.block_on(channel.send("HALT".to_string()).then(|r| {
        assert!(r.is_ok());
        assert_eq!(r.unwrap().unwrap().unwrap().downcast::<String>().unwrap().as_ref(), "HALT");
        Ok(()) as Result<(), ()>
    })).unwrap();

    // We shouldn't be able to send messages to it anymore
    assert!(runtime.connect("worker1").is_err());

    // But while we have an open connection, we can still use it
    r.block_on(channel.send("World2".to_string()).then(|r| {
        assert_eq!(r.unwrap().unwrap().unwrap().downcast::<String>().unwrap().as_ref(), "World2");
        Ok(()) as Result<(), ()>
    })).unwrap();
}
