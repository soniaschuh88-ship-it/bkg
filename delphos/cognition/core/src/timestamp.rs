use std::fmt; use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Serialize,Deserialize)]
#[serde(transparent)]
pub struct LogicalTimestamp(pub u64);
impl LogicalTimestamp {
    pub const ZERO:Self=Self(0);
    pub fn next(self)->Self{Self(self.0.checked_add(1).expect("overflow"))}
    pub fn as_u64(self)->u64{self.0}
}
impl From<u64> for LogicalTimestamp{fn from(v:u64)->Self{Self(v)}}
impl fmt::Display for LogicalTimestamp{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"t#{}",self.0)}}
#[cfg(test)] mod tests { use super::*;
    #[test] fn inc(){assert_eq!(LogicalTimestamp::ZERO.next(),LogicalTimestamp(1));}
    #[test] fn serde(){let t=LogicalTimestamp(42);assert_eq!(serde_json::to_string(&t).unwrap(),"42");}
}
