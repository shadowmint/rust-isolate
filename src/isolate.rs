use std::any::Any;
use crate::IsolateRuntime;
use crate::errors::isolate_error::IsolateError;
use futures::Future;

pub struct IsolateContext<'a> {
    pub identity: &'a str,
    pub runtime: &'a IsolateRuntime,
}

pub trait Isolate {
    fn handle(&self, input: Box<Any + Send + 'static>, context: IsolateContext) -> Box<dyn Future<Item=Result<Option<Box<Any + Send + 'static>>, IsolateError>, Error=IsolateError> + Send + 'static>;
}