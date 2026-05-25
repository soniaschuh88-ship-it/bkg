use ed25519_dalek::{Signature as DS,Signer,SigningKey,Verifier,VerifyingKey};
use rand::{rngs::OsRng,RngCore};
use serde::{Deserialize,Serialize};
use bkg_core::{BkgError,BkgResult,Signature};
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct PublicKey(pub [u8;32]);
impl PublicKey{
    pub fn from_bytes(b:[u8;32])->Self{Self(b)}
    pub fn as_bytes(&self)->&[u8;32]{&self.0}
    pub fn to_hex(&self)->String{hex::encode(self.0)}
    pub fn from_hex(s:&str)->BkgResult<Self>{let b=hex::decode(s).map_err(|e|BkgError::SignatureError(e.to_string()))?;if b.len()!=32{return Err(BkgError::SignatureError("pk=32 bytes".into()));}let mut a=[0u8;32];a.copy_from_slice(&b);Ok(Self(a))}
}
impl std::fmt::Display for PublicKey{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"{}",self.to_hex())}}
pub struct KeyPair{sk:SigningKey}
impl KeyPair{
    pub fn generate()->Self{let mut s=[0u8;32];OsRng.fill_bytes(&mut s);Self{sk:SigningKey::from_bytes(&s)}}
    pub fn from_seed(s:&[u8;32])->Self{Self{sk:SigningKey::from_bytes(s)}}
    pub fn public_key(&self)->PublicKey{PublicKey(self.sk.verifying_key().to_bytes())}
    pub fn secret_seed(&self)->[u8;32]{self.sk.to_bytes()}
    pub fn sign(&self,d:&[u8])->Signature{let s:DS=self.sk.sign(d);Signature::from_bytes(s.to_bytes().to_vec())}
}
impl std::fmt::Debug for KeyPair{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"KeyPair(pk={})",self.public_key().to_hex())}}
pub fn verify_signature(d:&[u8],sig:&Signature,pk:&PublicKey)->BkgResult<()>{
    let vk=VerifyingKey::from_bytes(pk.as_bytes()).map_err(|e|BkgError::SignatureError(e.to_string()))?;
    let sb:[u8;64]=sig.as_bytes().try_into().map_err(|_|BkgError::SignatureError(format!("sig must be 64 bytes, got {}",sig.as_bytes().len())))?;
    vk.verify(d,&DS::from_bytes(&sb)).map_err(|e|BkgError::SignatureError(e.to_string()))
}
#[cfg(test)]mod tests{use super::*;
    #[test]fn sv(){let k=KeyPair::generate();verify_signature(b"hi",&k.sign(b"hi"),&k.public_key()).unwrap();}
    #[test]fn tamper(){let k=KeyPair::generate();assert!(verify_signature(b"x",&k.sign(b"y"),&k.public_key()).is_err());}
    #[test]fn det(){let k1=KeyPair::from_seed(&[5u8;32]);let k2=KeyPair::from_seed(&[5u8;32]);assert_eq!(k1.public_key(),k2.public_key());}
}
