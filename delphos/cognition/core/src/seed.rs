use std::fmt;
use serde::{Deserialize,Serialize};
#[derive(Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct ExecutionSeed(pub [u8;32]);
impl ExecutionSeed {
    pub const ZERO:Self=Self([0u8;32]);
    pub fn random()->Self{let mut b=[0u8;32];getrandom::getrandom(&mut b).expect("CSPRNG");Self(b)}
    pub fn from_bytes(b:[u8;32])->Self{Self(b)}
    pub fn as_bytes(&self)->&[u8;32]{&self.0}
    pub fn to_hex(&self)->String{hex::encode(self.0)}
    pub fn from_hex(s:&str)->crate::BkgResult<Self>{let b=hex::decode(s).map_err(|e|crate::BkgError::Internal(format!("bad seed: {e}")))?;if b.len()!=32{return Err(crate::BkgError::Internal("seed=32 bytes".into()));}let mut a=[0u8;32];a.copy_from_slice(&b);Ok(Self(a))}
}
impl fmt::Debug for ExecutionSeed{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"Seed({:.16}...)",self.to_hex())}}
impl fmt::Display for ExecutionSeed{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"{}",self.to_hex())}}
#[cfg(test)] mod tests { use super::*;
    #[test] fn random_differs(){assert_ne!(ExecutionSeed::random(),ExecutionSeed::random());}
    #[test] fn hex_roundtrip(){let s=ExecutionSeed::random();assert_eq!(ExecutionSeed::from_hex(&s.to_hex()).unwrap(),s);}
}
