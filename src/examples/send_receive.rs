use crate::Isolate;
use crate::IsolateRuntime;
use futures::future::Future;
use std::any::Any;
use crate::errors::isolate_error::IsolateError;
use crate::IsolateTools;

struct Worker1 {}

impl Isolate for Worker1 {
    fn handle(&self, input: Box<Any + Send + 'static>, _: &IsolateRuntime) -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        let output = match input.downcast_ref::<String>() {
            Some(s) => self.handle_str(&s),
            None => "No input".to_string()
        };
        IsolateTools::some_as_future(output)
    }
}

impl Worker1 {
    fn handle_str(&self, input: &str) -> String {
        format!("Hello {}", input)
    }
}

#[test]
fn test_send_message_to_worker() {
    let mut r = tokio::runtime::Runtime::new().unwrap();
    let mut runtime = IsolateRuntime::new();

    runtime.set("worker1", Worker1 {}).unwrap();

    // Send on message
    {
        let channel = runtime.connect("worker1").unwrap();
        r.block_on(channel.send("World".to_string()).then(|r| {
            assert!(r.is_ok());
            assert_eq!(r.unwrap().unwrap().downcast::<String>().unwrap().as_ref(), "Hello World");
            Ok(()) as Result<(), ()>
        })).unwrap();
    }

    // Open a new channel and send a second message
    {
        let channel = runtime.connect("worker1").unwrap();
        r.block_on(channel.send("World2".to_string()).then(|r| {
            assert!(r.is_ok());
            assert_eq!(r.unwrap().unwrap().downcast::<String>().unwrap().as_ref(), "Hello World2");
            Ok(()) as Result<(), ()>
        })).unwrap();
    }
}
