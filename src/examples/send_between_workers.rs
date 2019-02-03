use crate::Isolate;
use crate::IsolateRuntime;
use futures::future::Future;
use std::any::Any;
use crate::errors::isolate_error::IsolateError;
use crate::IsolateTools;

// Worker1 delegates to worker2
struct Worker1 {}

impl Isolate for Worker1 {
    fn handle(&self, input: Box<Any + Send + 'static>, runtime: &IsolateRuntime) -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        match runtime.connect("worker2") {
            Ok(channel) => {
                Box::new(channel.send_boxed(input).then(|r| {
                    let output_string = r.unwrap().unwrap().downcast::<String>().unwrap().as_ref().to_string();
                    Ok(IsolateTools::some_as_box(output_string))
                }))
            }
            Err(_) => {
                IsolateTools::err_as_future(IsolateError::from("worker2 not available"))
            }
        }
    }
}

// Worker2 does the actual work
struct Worker2 {}

impl Isolate for Worker2 {
    fn handle(&self, input: Box<Any + Send + 'static>, _: &IsolateRuntime) -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        let output = match input.downcast_ref::<String>() {
            Some(s) => self.handle_str(&s),
            None => "No input".to_string()
        };
        IsolateTools::some_as_future(output)
    }
}

impl Worker2 {
    fn handle_str(&self, input: &str) -> String {
        format!("Hello {}", input)
    }
}

#[test]
fn test_send_message_to_worker() {
    let mut r = tokio::runtime::Runtime::new().unwrap();
    let mut runtime = IsolateRuntime::new();

    runtime.set("worker1", Worker1 {}).unwrap();
    runtime.set("worker2", Worker2 {}).unwrap();

    // Send on message
    let channel = runtime.connect("worker1").unwrap();
    r.block_on(channel.send("World".to_string()).then(|r| {
        assert!(r.is_ok());
        assert_eq!(r.unwrap().unwrap().downcast::<String>().unwrap().as_ref(), "Hello World");
        Ok(()) as Result<(), ()>
    })).unwrap();
}

#[test]
fn test_send_message_to_worker_fails_with_no_worker() {
    let mut r = tokio::runtime::Runtime::new().unwrap();
    let mut runtime = IsolateRuntime::new();

    runtime.set("worker1", Worker1 {}).unwrap();

    // Send on message
    let channel = runtime.connect("worker1").unwrap();
    r.block_on(channel.send("World".to_string()).then(|r| {
        assert!(r.is_err());
        println!("Final: {:?}", r);
        Ok(()) as Result<(), ()>
    })).unwrap();
}