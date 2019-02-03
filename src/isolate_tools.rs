use futures::Future;
use std::any::Any;
use crate::errors::isolate_error::IsolateError;
use std::error::Error;

pub struct IsolateTools
{}

impl IsolateTools {
    /// Return a boxed future for none
    pub fn none_as_future() -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        return Box::new(futures::finished(None));
    }

    /// Return a boxed result
    pub fn some_as_box<T: Any + Send + 'static>(response: T) -> Option<Box<Any + Send + 'static>> {
        let boxed_output = Box::new(response) as Box<Any + Send + 'static>;
        return Some(boxed_output);
    }

    /// Return a boxed future from a value
    pub fn some_as_future<T: Any + Send + 'static>(response: T) -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        let boxed_output = IsolateTools::some_as_box(response);
        Box::new(futures::finished(boxed_output))
    }

    /// Return a failure
    pub fn err_as_future(e: impl Error) -> Box<Future<Item=Option<Box<Any + Send + 'static>>, Error=IsolateError> + Send + 'static> {
        let err = IsolateError::from(e);
        Box::new(futures::failed(err))
    }
}