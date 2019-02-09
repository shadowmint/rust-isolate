pub(crate) mod isolate_identity;
pub(crate) mod isolate_runtime_error;
mod isolate_runner;

#[derive(Clone)]
pub struct IsolateRuntime {}

impl IsolateRuntime {
    pub fn new() -> IsolateRuntime {
        IsolateRuntime {}
    }

    pub fn lookup(name: &str) -> Option<Isolate>
    pub fn open()
}