use std::fmt::Display;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum IsolateRegistryError {
    InternalSyncError,
    IdentityAlreadyInUse,
    NoMatchingIdentity,
    InvalidRuntimeType
}

impl Error for IsolateRegistryError {}

impl Display for IsolateRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}