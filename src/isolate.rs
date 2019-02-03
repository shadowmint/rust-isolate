use std::any::Any;
use crate::IsolateRuntime;
use crate::errors::isolate_error::IsolateError;
use futures::Future;

pub trait Isolate {
    fn handle(&self, input: Box<Any + Send + 'static>, runtime: &IsolateRuntime) -> Box<dyn Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static>;
}