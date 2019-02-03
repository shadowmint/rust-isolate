use crossbeam::channel::Receiver;
use std::any::Any;
use crate::isolate::Isolate;
use crate::isolate_runtime::IsolateRuntime;
use futures::sync::oneshot;
use crate::errors::isolate_runtime_error::IsolateRuntimeError;
use futures::Future;

pub struct IsolateRuntimeInstance {
    receiver: Receiver<(Box<Any + Send + 'static>, oneshot::Sender<Option<Box<Any + Send + 'static>>>)>,
    isolate: Box<dyn Isolate + Send + 'static>,
    runtime: IsolateRuntime,
}

impl IsolateRuntimeInstance {
    pub fn new(receiver: Receiver<(Box<Any + Send + 'static>, oneshot::Sender<Option<Box<Any + Send + 'static>>>)>, isolate: impl Isolate + Send + 'static, runtime: IsolateRuntime) -> IsolateRuntimeInstance {
        return IsolateRuntimeInstance {
            receiver,
            isolate: Box::new(isolate),
            runtime,
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
                tokio::spawn(self.isolate.handle(input, &self.runtime).then(move |result| {
                    match result {
                        Ok(output) => {
                            match response.send(output) {
                                Ok(_) => {}
                                Err(e) => {
                                    // TODO: What do with errors?
                                    println!("{:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            // TODO: What do with errors?
                            println!("{:?}", e);
                        }
                    }
                    Ok(()) as Result<(), ()>
                }));
                Ok(())
            }
            Err(_) => Err(IsolateRuntimeError::IsolateHalted)
        }
    }
}
