use crossbeam::{unbounded, Receiver, Sender};

/// IsolateChannel wraps a multi-producer multi-consumer channel that can be safely passed between
/// threads; it is safe to clone and share this object, but realize it basically acts as a RC on
/// the isolate instance.
pub struct IsolateChannel<T: Send + 'static> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}

impl<T: Send + 'static> IsolateChannel<T> {
    /// Create a new instance
    pub fn new() -> (IsolateChannel<T>, IsolateChannel<T>) {
        let (runner_s, runner_r) = unbounded();
        let (worker_s, worker_r) = unbounded();
        (
            IsolateChannel {
                sender: worker_s,
                receiver: runner_r,
            },
            IsolateChannel {
                sender: runner_s,
                receiver: worker_r,
            },
        )
    }

    /// Clone the references in this instance
    pub fn clone(&self) -> IsolateChannel<T> {
        return IsolateChannel {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::IsolateChannel;

    #[test]
    pub fn test_new_channel() {
        let _ = IsolateChannel::<String>::new();
    }
}
