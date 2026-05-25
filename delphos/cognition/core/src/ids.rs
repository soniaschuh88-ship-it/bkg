use std::{fmt,str::FromStr};
use serde::{Deserialize,Serialize};
use uuid::Uuid;
use crate::BkgError;
macro_rules! typed_id { ($name:ident) => {
    #[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize,PartialOrd,Ord)]
    #[serde(transparent)]
    pub struct $name(pub Uuid);
    impl $name {
        pub fn new() -> Self { Self(Uuid::new_v4()) }
        pub fn as_uuid(self) -> Uuid { self.0 }
    }
    impl Default for $name { fn default() -> Self { Self::new() } }
    impl fmt::Display for $name { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f,"{}",self.0.hyphenated()) } }
    impl FromStr for $name { type Err=BkgError; fn from_str(s:&str)->Result<Self,Self::Err> { Uuid::parse_str(s).map(Self).map_err(|e|BkgError::Internal(format!("invalid {}: {e}",stringify!($name)))) } }
}; }
typed_id!(AgentId); typed_id!(CapsuleId); typed_id!(EventId);
typed_id!(ContractId); typed_id!(SessionId); typed_id!(TaskId);
#[cfg(test)] mod tests { use super::*;
    #[test] fn roundtrip() { let id=AgentId::new(); assert_eq!(id.to_string().parse::<AgentId>().unwrap(),id); }
    #[test] fn serde() { let id=EventId::new(); let j=serde_json::to_string(&id).unwrap(); assert!(j.starts_with('"')); assert_eq!(serde_json::from_str::<EventId>(&j).unwrap(),id); }
}
