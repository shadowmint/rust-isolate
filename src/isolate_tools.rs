use futures::Future;
use std::any::Any;
use crate::errors::isolate_error::IsolateError;

pub struct IsolateTools
{}

impl IsolateTools {
    /// Return a boxed future for none
    pub fn none() -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        return Box::new(futures::finished(None));
    }

    /// Return a boxed future from a value
    pub fn response<T: Any + Send + 'static>(response: Result<T, IsolateError>) -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        match response {
            Ok(r) => {
                let boxed_output = IsolateTools::some(r);
                Box::new(futures::finished(boxed_output))
            }
            Err(e) => Box::new(futures::failed(e))
        }
    }

    pub fn some<T: Any + Send + 'static>(response: T) -> Option<Box<Any + Send + 'static>> {
        let boxed_output = Box::new(response) as Box<Any + Send + 'static>;
        return Some(boxed_output);
    }
}