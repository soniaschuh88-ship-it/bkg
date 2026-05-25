use std::path::PathBuf;
use bkg_core::{BkgError,BkgResult,ContractId,EventId,ExecutionSeed,LogicalTimestamp,RealmId,SessionId};
use bkg_crypto::{hash::hash_bytes,signing::KeyPair};
use crate::{ops::{ReadOp,WriteOp},swd::{Swd,VerificationResult},trace::RealmTrace};

pub struct SwdEngine{active:Option<Swd>,archive:Vec<Swd>}
impl SwdEngine{
    pub fn new()->Self{Self{active:None,archive:Vec::new()}}
    pub fn init(&mut self,sid:SessionId,realm:RealmId,seed:ExecutionSeed,input:&[u8])->BkgResult<()>{
        if self.active.is_some(){return Err(BkgError::Internal("active session exists".into()));}
        self.active=Some(Swd{session_id:sid,initiating_realm:realm,input_hash:hash_bytes(input),execution_seed:seed,realm_trace:Vec::new(),contracts:Vec::new(),touched_files:Vec::new(),write_ops:Vec::new(),read_ops:Vec::new(),provider_trace:Vec::new(),budget_used:0,event_range_start:None,event_range_end:None,hash_root:bkg_core::Hash256::ZERO,signature:None,verification_result:None,started_at:chrono::Utc::now(),committed_at:None});
        Ok(())
    }
    fn swd(&mut self)->BkgResult<&mut Swd>{self.active.as_mut().ok_or_else(||BkgError::Internal("no active session".into()))}
    pub fn capture_write(&mut self,op:WriteOp)->BkgResult<()>{if let Some(ref p)=op.file_path.clone(){let s=self.swd()?;if!s.touched_files.contains(p){s.touched_files.push(p.clone());}}self.swd()?.write_ops.push(op);Ok(())}
    pub fn capture_read(&mut self,op:ReadOp)->BkgResult<()>{self.swd()?.read_ops.push(op);Ok(())}
    pub fn capture_realm_enter(&mut self,r:RealmId,t:LogicalTimestamp,l:impl Into<String>)->BkgResult<()>{let tr=RealmTrace::enter(r,t,l);self.swd()?.realm_trace.push(tr);Ok(())}
    pub fn capture_realm_exit(&mut self,t:LogicalTimestamp)->BkgResult<()>{if let Some(last)=self.swd()?.realm_trace.last_mut(){last.exit(t);}Ok(())}
    pub fn capture_contract(&mut self,id:ContractId)->BkgResult<()>{self.swd()?.contracts.push(id);Ok(())}
    pub fn capture_event(&mut self,id:EventId)->BkgResult<()>{let s=self.swd()?;if s.event_range_start.is_none(){s.event_range_start=Some(id);}s.event_range_end=Some(id);Ok(())}
    pub fn capture_provider(&mut self,e:impl Into<String>)->BkgResult<()>{self.swd()?.provider_trace.push(e.into());Ok(())}
    pub fn add_budget(&mut self,n:u64)->BkgResult<()>{self.swd()?.budget_used+=n;Ok(())}
    pub fn touch_file(&mut self,p:PathBuf)->BkgResult<()>{let s=self.swd()?;if!s.touched_files.contains(&p){s.touched_files.push(p);}Ok(())}
    pub fn commit(&mut self,kp:Option<&KeyPair>)->BkgResult<Swd>{
        let mut swd=self.active.take().ok_or_else(||BkgError::Internal("no session".into()))?;
        swd.hash_root=swd.compute_hash_root();
        swd.committed_at=Some(chrono::Utc::now());
        if let Some(k)=kp{swd.signature=Some(k.sign(swd.hash_root.as_bytes()));}
        swd.verification_result=Some(VerificationResult::ok());
        self.archive.push(swd.clone());Ok(swd)
    }
    pub fn verify(swd:&Swd,pk:Option<&bkg_crypto::PublicKey>)->VerificationResult{
        if !swd.verify_hash_root(){return VerificationResult::failed("hash-root mismatch");}
        if let Some(sig)=&swd.signature{match pk{Some(k)=>if bkg_crypto::verify_signature(swd.hash_root.as_bytes(),sig,k).is_err(){return VerificationResult::failed("bad sig");},None=>return VerificationResult{hash_valid:true,signature_valid:false,detail:Some("no pk".into())},}}
        VerificationResult::ok()
    }
    pub fn archived(&self)->&[Swd]{&self.archive}
    pub fn is_active(&self)->bool{self.active.is_some()}
}
impl Default for SwdEngine{fn default()->Self{Self::new()}}

#[cfg(test)]
mod tests{
    use super::*;
    use bkg_core::{ExecutionSeed,RealmId,SessionId};
    fn start(e:&mut SwdEngine)->SessionId{let s=SessionId::new();e.init(s,RealmId::Telum,ExecutionSeed::random(),b"t").unwrap();s}
    #[test]fn commit_ok(){let mut e=SwdEngine::new();start(&mut e);let s=e.commit(None).unwrap();assert!(s.is_committed());assert!(s.verify_hash_root());}
    #[test]fn double_init_fails(){let mut e=SwdEngine::new();start(&mut e);assert!(e.init(SessionId::new(),RealmId::Styx,ExecutionSeed::random(),b"x").is_err());}
    #[test]fn no_session_fails(){let mut e=SwdEngine::new();assert!(e.commit(None).is_err());}
    #[test]fn ops(){let mut e=SwdEngine::new();start(&mut e);e.capture_write(WriteOp::new(LogicalTimestamp(1),RealmId::Causa,"op")).unwrap();e.capture_read(ReadOp::new(LogicalTimestamp(2),RealmId::Mensa,"get","k")).unwrap();let s=e.commit(None).unwrap();assert_eq!(s.op_count(),2);assert!(s.verify_hash_root());}
    #[test]fn sign_verify(){let kp=bkg_crypto::signing::KeyPair::generate();let pk=kp.public_key();let mut e=SwdEngine::new();start(&mut e);let s=e.commit(Some(&kp)).unwrap();assert!(SwdEngine::verify(&s,Some(&pk)).is_valid());}
    #[test]fn tamper(){let mut e=SwdEngine::new();start(&mut e);e.capture_write(WriteOp::new(LogicalTimestamp(1),RealmId::Causa,"op")).unwrap();let mut s=e.commit(None).unwrap();s.write_ops.push(WriteOp::new(LogicalTimestamp(99),RealmId::Styx,"inj"));assert!(!s.verify_hash_root());}
    #[test]fn archive(){let mut e=SwdEngine::new();for _ in 0..3{start(&mut e);e.commit(None).unwrap();}assert_eq!(e.archived().len(),3);}
}
