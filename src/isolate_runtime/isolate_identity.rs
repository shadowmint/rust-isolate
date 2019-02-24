use uuid::Uuid;
use crate::IsolateRuntimeError;
use std::error::Error;
use std::fmt::Display;
use std::fmt;

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

    pub fn to_string(&self) -> String {
        self.identity.to_string()
    }

    pub fn try_from(value: &str) -> Result<IsolateIdentity, IsolateRuntimeError> {
        match Uuid::parse_str(value) {
            Ok(id) => {
                Ok(IsolateIdentity {
                    identity: id
                })
            }
            Err(e) => {
                Err(IsolateRuntimeError::InvalidIdentity(e.description().to_string()))
            }
        }
    }
}

impl Display for IsolateIdentity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
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

    #[test]
    pub fn test_isolate_identity_serialization() {
        let a = IsolateIdentity::new();
        let b = a.to_string();
        let c = IsolateIdentity::try_from(&b).unwrap();

        assert_eq!(a, c);
    }
}
