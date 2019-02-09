use uuid::Uuid;
use std::hash::Hash;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct IsolateIdentity {
    identity: Uuid
}

impl IsolateIdentity {
    pub fn new() -> IsolateIdentity {
        IsolateIdentity {
            identity: Uuid::new_v4()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::IsolateIdentity;

    #[test]
    pub fn test_isolate_identity_equivalence() {
        let a = IsolateIdentity::new();
        let b = a;
        let c = IsolateIdentity::new();

        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}