pub trait IsolateRuntimeWait {
    /// Wait for all runtime handles
    fn wait(&self);
}