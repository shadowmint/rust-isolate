use crossbeam::{Sender, Receiver, unbounded};
use std::sync::Arc;
use std::sync::Mutex;

/// IsolateChannel wraps a multi-producer multi-consumer channel that can be safely passed between
/// threads; it is safe to clone and share this object, but realize it basically acts as a RC on
/// the isolate instance.
pub struct IsolateChannel<T: Send + 'static> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
    active: Arc<Mutex<bool>>,
}

impl<T: Send + 'static> IsolateChannel<T> {
    /// Create a new instance
    pub fn new() -> (IsolateChannel<T>, IsolateChannel<T>) {
        let active = Arc::new(Mutex::new(true));
        let (runner_s, runner_r) = unbounded();
        let (worker_s, worker_r) = unbounded();
        (IsolateChannel {
            sender: worker_s,
            receiver: runner_r,
            active: active.clone(),
        }, IsolateChannel {
            sender: runner_s,
            receiver: worker_r,
            active,
        })
    }

    /// Clone this channel if it is not closed
    pub fn clone(&self) -> Option<IsolateChannel<T>> {
        match self.active.lock() {
            Ok(active) => {
                if *active {
                    Some(IsolateChannel {
                        sender: self.sender.clone(),
                        receiver: self.receiver.clone(),
                        active: self.active.clone(),
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Mark this channel as closed so it can't be cloned any more.
    pub fn close(self) {
        match self.active.lock() {
            Ok(mut active) => { *active = false; }
            Err(_) => {}
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

    #[test]
    pub fn test_cant_clone_closed_channel() {
        let (a, b) = IsolateChannel::<String>::new();
        let c = a.clone().unwrap();

        a.close();
        assert!(b.clone().is_none());
    }
}

