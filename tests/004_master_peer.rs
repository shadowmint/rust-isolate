use crate::peer::PeerIsolate;
use crate::master::MasterIsolate;
use rust_isolate::IsolateRegistry;
use crate::master::MasterEvent;
use crate::peer::PeerEvent;
use std::thread;
use std::time::Duration;

// In this example, we register are connecting a number of peers to a single master.
// We push events to the peers, who push different internal events to the master.
// The master pushes changes back to the peers, who then push it back externally.

mod peer {
    use rust_isolate::Isolate;
    use rust_isolate::IsolateIdentity;
    use rust_isolate::IsolateChannel;
    use rust_isolate::IsolateRegistryRef;
    use crate::master::MasterEvent;

    pub struct PeerIsolate {
        pub identity: IsolateIdentity,
        registry: IsolateRegistryRef,
        channel: Option<IsolateChannel<PeerEvent>>,
        master: Option<IsolateChannel<MasterEvent>>,
    }

    #[derive(Debug)]
    pub enum PeerEvent {
        Halt,
        Initialize(IsolateIdentity),
        Add(usize, usize),
        Sub(usize, usize),
        Mul(usize, usize),
        Output(isize),
    }

    impl Clone for PeerIsolate {
        fn clone(&self) -> Self {
            PeerIsolate::new(self.registry.clone())
        }
    }

    impl PeerEvent {
        pub fn as_output(&self) -> isize {
            match self {
                PeerEvent::Output(s) => *s,
                _ => unreachable!()
            }
        }
    }

    impl PeerIsolate {
        pub fn new(registry: IsolateRegistryRef) -> PeerIsolate {
            PeerIsolate {
                registry,
                identity: IsolateIdentity::new(),
                master: None,
                channel: None,
            }
        }

        pub fn dispatch(&mut self, event: PeerEvent) -> Result<(), ()> {
            println!("Peer event: {:?}", event);
            match event {
                PeerEvent::Halt => Err(()),
                PeerEvent::Add(a, b) => self.master_request(PeerEvent::Add(a, b)),
                PeerEvent::Sub(a, b) => self.master_request(PeerEvent::Sub(a, b)),
                PeerEvent::Mul(a, b) => self.master_request(PeerEvent::Mul(a, b)),
                PeerEvent::Output(s) => self.handle_output(s),
                PeerEvent::Initialize(id) => self.initialize(id),
                _ => Err(()),
            }
        }

        pub fn event_loop(&mut self, channel: &IsolateChannel<PeerEvent>) -> Result<(), ()> {
            self.channel = channel.clone();
            loop {
                match channel.receiver.recv() {
                    Ok(event) => { self.dispatch(event)?; }
                    Err(_err) => { return Err(()); }
                }
            }
        }

        fn handle_output(&self, output: isize) -> Result<(), ()> {
            self.channel.as_ref().unwrap().sender.send(PeerEvent::Output(output)).unwrap();
            Ok(())
        }

        fn initialize(&mut self, id: IsolateIdentity) -> Result<(), ()> {
            let runtime = self.registry.find::<MasterEvent>("Master").unwrap();
            let instance = runtime.find(&id).unwrap();
            println!("Sending peer to master");
            instance.sender.send(MasterEvent::NewPeer(self.identity.clone())).unwrap();
            self.master = Some(instance);
            Ok(())
        }

        fn master_request(&mut self, request: PeerEvent) -> Result<(), ()> {
            match self.master.as_ref() {
                Some(master_channel) => {
                    // Pass the request on to the master; the master has already got a copy of our
                    // channel, so now we just wait for a response from the master.
                    master_channel.sender.send(MasterEvent::PeerQueryRequest(self.identity.clone(), request)).unwrap();
                }
                None => { /* Not connected */ }
            }
            Ok(())
        }
    }

    impl Isolate<PeerEvent> for PeerIsolate {
        fn spawn(&self, identity: IsolateIdentity, channel: IsolateChannel<PeerEvent>) -> Box<FnMut() + Send + 'static> {
            let mut instance = self.clone();
            instance.identity = identity;
            Box::new(move || {
                let _ = instance.event_loop(&channel);
            })
        }
    }
}

mod master {
    use rust_isolate::Isolate;
    use rust_isolate::IsolateIdentity;
    use rust_isolate::IsolateChannel;
    use rust_isolate::IsolateRegistryRef;
    use crate::peer::PeerEvent;
    use std::collections::HashMap;

    pub struct MasterIsolate {
        peers: HashMap<IsolateIdentity, IsolateChannel<PeerEvent>>,
        registry: IsolateRegistryRef,
    }

    #[derive(Debug)]
    pub enum MasterEvent {
        Halt,
        MasterIdentity(IsolateIdentity),
        NewPeer(IsolateIdentity),
        PeerQueryRequest(IsolateIdentity, PeerEvent),
        PeerQueryResponse(isize),
    }

    impl MasterEvent {
        pub fn identity(&self) -> IsolateIdentity {
            match self {
                MasterEvent::MasterIdentity(id) => id.clone(),
                _ => unreachable!()
            }
        }
    }

    impl Clone for MasterIsolate {
        fn clone(&self) -> Self {
            return MasterIsolate {
                peers: HashMap::new(),
                registry: self.registry.clone(),
            };
        }
    }

    impl MasterIsolate {
        pub fn new(registry: IsolateRegistryRef) -> MasterIsolate {
            MasterIsolate {
                peers: HashMap::new(),
                registry,
            }
        }

        pub fn dispatch(&mut self, event: MasterEvent) -> Result<(), ()> {
            println!("Master event: {:?}", event);
            match event {
                MasterEvent::Halt => Err(()),
                MasterEvent::NewPeer(id) => self.peer_connected(id),
                MasterEvent::PeerQueryRequest(id, req) => self.peer_request(id, req),
                _ => Err(()),
            }
        }

        pub fn event_loop(&mut self, channel: &IsolateChannel<MasterEvent>) -> Result<(), ()> {
            loop {
                match channel.receiver.recv() {
                    Ok(event) => { self.dispatch(event)?; }
                    Err(_err) => { return Err(()); }
                }
            }
        }

        fn peer_connected(&mut self, id: IsolateIdentity) -> Result<(), ()> {
            let runtime = self.registry.find::<PeerEvent>("Peer").unwrap();
            let instance = runtime.find(&id).unwrap();
            self.peers.insert(id, instance);
            Ok(())
        }

        fn peer_request(&mut self, id: IsolateIdentity, req: PeerEvent) -> Result<(), ()> {
            match self.peers.get(&id) {
                Some(peer_ref) => {
                    match req {
                        PeerEvent::Add(a, b) => {
                            peer_ref.sender.send(PeerEvent::Output(a as isize + b as isize)).unwrap()
                        }
                        PeerEvent::Mul(a, b) => {
                            peer_ref.sender.send(PeerEvent::Output(a as isize * b as isize)).unwrap()
                        }
                        PeerEvent::Sub(a, b) => {
                            peer_ref.sender.send(PeerEvent::Output(a as isize - b as isize)).unwrap()
                        }
                        _ => {}
                    }
                }
                None => {
                    println!("Ignoring request from invalid client id");
                }
            }
            Ok(())
        }
    }

    impl Isolate<MasterEvent> for MasterIsolate {
        fn spawn(&self, identity: IsolateIdentity, channel: IsolateChannel<MasterEvent>) -> Box<FnMut() + Send + 'static> {
            let mut instance = self.clone();
            Box::new(move || {
                channel.sender.send(MasterEvent::MasterIdentity(identity)).unwrap();
                let _ = instance.event_loop(&channel);
            })
        }
    }
}

#[test]
pub fn main() {
    let mut registry = IsolateRegistry::new();
    let mut peers = registry.bind("Peer", PeerIsolate::new(registry.as_ref())).unwrap();
    let mut masters = registry.bind("Master", MasterIsolate::new(registry.as_ref())).unwrap();

    // Create a master instance
    let master = masters.spawn().unwrap();
    let master_identity = master.receiver.recv().unwrap().identity();

    // Create a set of peers and notify them of the master instance id
    let peer1 = peers.spawn().unwrap();
    let peer2 = peers.spawn().unwrap();
    let peer3 = peers.spawn().unwrap();
    peer1.sender.send(PeerEvent::Initialize(master_identity.clone())).unwrap();
    peer2.sender.send(PeerEvent::Initialize(master_identity.clone())).unwrap();
    peer3.sender.send(PeerEvent::Initialize(master_identity.clone())).unwrap();

    // Now we push events to the peers and the master should process and respond to them
    peer1.sender.send(PeerEvent::Add(1, 1)).unwrap();
    peer2.sender.send(PeerEvent::Sub(10, 15)).unwrap();
    peer3.sender.send(PeerEvent::Mul(0, 50)).unwrap();

    // And finally we should get a response back from the peers
    assert_eq!(peer1.receiver.recv().unwrap().as_output(), 2);
    assert_eq!(peer2.receiver.recv().unwrap().as_output(), -5);
    assert_eq!(peer3.receiver.recv().unwrap().as_output(), 0);

    // Halt everyone
    master.sender.send(MasterEvent::Halt).unwrap();
    peer1.sender.send(PeerEvent::Halt).unwrap();
    peer2.sender.send(PeerEvent::Halt).unwrap();
    peer3.sender.send(PeerEvent::Halt).unwrap();

    registry.wait();
}