use crossbeam::channel::Receiver;
use std::any::Any;
use crate::isolate::Isolate;
use crate::isolate_runtime::IsolateRuntime;
use futures::sync::oneshot;
use crate::errors::isolate_runtime_error::IsolateRuntimeError;
use futures::Future;
use crate::errors::isolate_error::IsolateError;
use crate::isolate::IsolateContext;

pub struct IsolateRuntimeInstance {
    receiver: Receiver<(Box<Any + Send + 'static>, oneshot::Sender<Result<Option<Box<Any + Send + 'static>>, IsolateError>>)>,
    isolate: Box<dyn Isolate + Send + 'static>,
    runtime: IsolateRuntime,
    identity: String,
}

impl IsolateRuntimeInstance {
    pub fn new(identity: &str, receiver: Receiver<(Box<Any + Send + 'static>, oneshot::Sender<Result<Option<Box<Any + Send + 'static>>, IsolateError>>)>, isolate: impl Isolate + Send + 'static, runtime: IsolateRuntime) -> IsolateRuntimeInstance {
        return IsolateRuntimeInstance {
            receiver,
            isolate: Box::new(isolate),
            runtime,
            identity: identity.to_string(),
        };
    }

    pub fn run(self) {
        tokio::run(futures::lazy(move || {
            loop {
                match self.event_loop() {
                    Ok(_) => {}
                    Err(e) => {
                        match e {
                            IsolateRuntimeError::IsolateHalted => { break; }
                            _ => { /* Do nothing for others */ }
                        }
                    }
                }
            }
            Ok(())
        }));
    }

    fn event_loop(&self) -> Result<(), IsolateRuntimeError> {
        match self.receiver.recv() {
            Ok((input, response)) => {
                tokio::spawn(self.isolate.handle(input, self.new_context()).then(move |result| {
                    match result {
                        Ok(output) => {
                            match response.send(output) {
                                Ok(_) => {}
                                Err(e) => {
                                    // TODO: What do with errors?
                                    println!("Failed to send response: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            match response.send(Err(e)) {
                                Ok(_) => {}
                                Err(e) => {
                                    // TODO: What do with errors?
                                    println!("Failed to send response: {:?}", e);
                                }
                            }
                        }
                    }
                    Ok(()) as Result<(), ()>
                }));
                Ok(())
            }
            Err(_) => Err(IsolateRuntimeError::IsolateHalted)
        }
    }

    fn new_context(&self) -> IsolateContext {
        IsolateContext {
            runtime: &self.runtime,
            identity: self.identity.as_ref(),
        }
    }
}
