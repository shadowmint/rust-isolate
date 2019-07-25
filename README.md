# rust-isolate

Isolated worker processes for rust.

## version

Until this migrates to `std::future` it'll stay pre-1.0.0

## usage

See the `tests` folder for examples of usage.

Example:

```
use rust_isolate::Isolate;
use rust_isolate::IsolateIdentity;
use rust_isolate::IsolateChannel;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Duration;
use std::thread;
use rust_isolate::IsolateRegistry;
use rust_isolate::IsolateRegistryRef;


#[derive(Debug)]
enum ChatMessage {
    Halt,
    BroadcastMessage(String),
    NewMessage(String),
}

struct ChatServer {
    pub connections: HashMap<IsolateIdentity, IsolateChannel<ChatMessage>>
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {
            connections: HashMap::new()
        }
    }

    pub fn broadcast(&self, message: String) {
        self.connections.iter().for_each(|(_, v)| {
            v.sender.send(ChatMessage::BroadcastMessage(message.clone())).unwrap()
        })
    }
}

struct ChatService {
    pub registry: IsolateRegistryRef,
    pub server: Arc<Mutex<ChatServer>>,
}

impl ChatService {
    pub fn new(registry: IsolateRegistryRef) -> ChatService {
        ChatService {
            registry,
            server: Arc::new(Mutex::new(ChatServer::new())),
        }
    }
}

impl Isolate<ChatMessage> for ChatService {
    fn spawn(&self, identity: IsolateIdentity, channel: IsolateChannel<ChatMessage>) -> Box<FnMut() + Send + 'static> {
        let server = self.server.clone();
        let registry = self.registry.clone();
        Box::new(move || {
            {
                let mut server_ref = server.lock().unwrap();
                let runtime = registry.find("Chat").unwrap();
                let self_pointing_channel = runtime.find(&identity).unwrap();
                server_ref.connections.insert(identity, self_pointing_channel);
            }
            loop {
                match channel.receiver.recv_timeout(Duration::from_millis(1000)) {
                    Ok(r) => {
                        match r {
                            ChatMessage::Halt => {
                                break;
                            }
                            // A message from the external client to send a new chat message
                            ChatMessage::NewMessage(s) => {
                                let server_ref = server.lock().unwrap();
                                server_ref.broadcast(s);
                            }

                            // A message back from the broadcaster, to post to the client
                            ChatMessage::BroadcastMessage(s) => {
                                channel.sender.send(ChatMessage::BroadcastMessage(s)).unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        if !e.is_timeout() {
                            break;
                        }
                    }
                }
            }
            {
                let mut server_ref = server.lock().unwrap();
                server_ref.connections.remove(&identity);
            }
        })
    }
}

#[test]
pub fn main() {
    let mut registry = IsolateRegistry::new();
    let mut runtime = registry.bind("Chat", ChatService::new(registry.as_ref())).unwrap();

    let c1 = runtime.spawn().unwrap();
    let c2 = runtime.spawn().unwrap();
    let c3 = runtime.spawn().unwrap();

    // Wait for all remote threads to startup
    thread::sleep(Duration::from_millis(100));

    c1.sender.send(ChatMessage::NewMessage("Hello World".to_string())).unwrap();

    match c1.receiver.recv().unwrap() {
        ChatMessage::BroadcastMessage(c) => { assert_eq!("Hello World", c); }
        _ => unreachable!()
    }

    match c2.receiver.recv().unwrap() {
        ChatMessage::BroadcastMessage(c) => { assert_eq!("Hello World", c); }
        _ => unreachable!()
    }

    match c3.receiver.recv().unwrap() {
        ChatMessage::BroadcastMessage(c) => { assert_eq!("Hello World", c); }
        _ => unreachable!()
    }

    c1.sender.send(ChatMessage::Halt).unwrap();
    c2.sender.send(ChatMessage::Halt).unwrap();
    c3.sender.send(ChatMessage::Halt).unwrap();

    registry.wait();
}
```
