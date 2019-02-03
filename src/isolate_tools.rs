use futures::Future;
use std::any::Any;
use crate::errors::isolate_error::IsolateError;
use std::error::Error;

pub struct IsolateTools
{}

impl IsolateTools {
    /// Return a boxed future for none
    pub fn none_as_future() -> Box<Future<Item=Result<Option<Box<Any + Send + 'static>>, IsolateError>, Error=IsolateError> + Send + 'static> {
        return Box::new(futures::finished(Ok(None)));
    }

    /// Return a boxed result
    pub fn some_as_box<T: Any + Send + 'static>(response: T) -> Result<Option<Box<Any + Send + 'static>>, IsolateError> {
        let boxed_output = Box::new(response) as Box<Any + Send + 'static>;
        return Ok(Some(boxed_output));
    }

    /// Return a boxed future from a value
    pub fn some_as_future<T: Any + Send + 'static>(response: T) -> Box<Future<Item=Result<Option<Box<Any + Send + 'static>>, IsolateError>, Error=IsolateError> + Send + 'static> {
        let boxed_output = IsolateTools::some_as_box(response);
        Box::new(futures::finished(boxed_output))
    }

    /// Return a failure
    pub fn err_as_future(e: impl Error) -> Box<Future<Item=Result<Option<Box<Any + Send + 'static>>, IsolateError>, Error=IsolateError> + Send + 'static> {
        let err = IsolateError::from_error(e);
        Box::new(futures::failed(err))
    }

    /// Return a failure
    pub fn isolate_err_as_future(e: IsolateError) -> Box<Future<Item=Result<Option<Box<Any + Send + 'static>>, IsolateError>, Error=IsolateError> + Send + 'static> {
        Box::new(futures::failed(e))
    }
}