use std::fmt;
use serde::{Deserialize,Serialize};
use crate::BkgError;
#[derive(Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct Hash256(pub [u8;32]);
impl Hash256 {
    pub const ZERO:Self=Self([0u8;32]);
    pub fn as_bytes(&self)->&[u8;32]{&self.0}
    pub fn to_hex(&self)->String{hex::encode(self.0)}
    pub fn is_zero(&self)->bool{self==&Self::ZERO}
    pub fn from_hex(s:&str)->crate::BkgResult<Self>{let b=hex::decode(s).map_err(|e|BkgError::InvalidHash(e.to_string()))?;if b.len()!=32{return Err(BkgError::InvalidHash(format!("need 32 bytes, got {}",b.len())));}let mut a=[0u8;32];a.copy_from_slice(&b);Ok(Self(a))}
}
impl fmt::Debug for Hash256{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"Hash256({})",self.to_hex())}}
impl fmt::Display for Hash256{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"{}",self.to_hex())}}
impl std::str::FromStr for Hash256{type Err=BkgError;fn from_str(s:&str)->Result<Self,Self::Err>{Self::from_hex(s)}}
#[derive(Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Signature(pub Vec<u8>);
impl Signature{pub fn from_bytes(b:Vec<u8>)->Self{Self(b)}pub fn as_bytes(&self)->&[u8]{&self.0}pub fn to_hex(&self)->String{hex::encode(&self.0)}}
impl fmt::Debug for Signature{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"Sig({})",self.to_hex())}}
#[cfg(test)] mod tests { use super::*;
    #[test] fn hex_roundtrip(){let h=Hash256([0xABu8;32]);assert_eq!(Hash256::from_hex(&h.to_hex()).unwrap(),h);}
    #[test] fn zero(){assert!(Hash256::ZERO.is_zero());assert!(!Hash256([1u8;32]).is_zero());}
}
