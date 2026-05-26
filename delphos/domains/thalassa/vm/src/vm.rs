use std::collections::HashMap;
use serde::{Deserialize,Serialize};
use bkg_core::{BkgError,BkgResult};
use crate::{limits::ResourceLimits,mount::VfsMount,process::VmProcess,snapshot::VmSnapshot,syscall::SyscallFilter};

#[derive(Debug,Clone,thiserror::Error)]
pub enum VmError{
    #[error("resource limit exceeded: {0}")] LimitExceeded(String),
    #[error("syscall denied: {0}")] SyscallDenied(String),
    #[error("path not mounted: {0}")] PathNotMounted(String),
    #[error("vm sealed — no further execution")] Sealed,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize,Default)]
#[serde(rename_all="snake_case")]
pub enum VmState{#[default]Idle,Running,Paused,Sealed,Error}

pub struct SandboxVm{
    pub id:String, pub limits:ResourceLimits,
    pub mounts:Vec<VfsMount>, pub env:HashMap<String,String>,
    pub working_dir:String, pub filter:SyscallFilter,
    pub state:VmState, pub processes:Vec<VmProcess>,
    pub snapshots:Vec<VmSnapshot>,
}
impl SandboxVm{
    pub fn new(limits:ResourceLimits)->Self{
        Self{id:uuid::Uuid::new_v4().to_string(),limits,mounts:vec![],env:HashMap::new(),working_dir:"/workspace".into(),filter:SyscallFilter::minimal(),state:VmState::Idle,processes:vec![],snapshots:vec![]}
    }
    pub fn add_mount(&mut self,m:VfsMount){self.mounts.push(m);}
    pub fn set_env(&mut self,k:impl Into<String>,v:impl Into<String>){self.env.insert(k.into(),v.into());}
    pub fn snapshot(&mut self,label:&str)->VmSnapshot{
        let snap=VmSnapshot::capture(&self.id,label,self.limits.clone(),self.mounts.clone(),self.env.clone(),self.working_dir.clone());
        self.snapshots.push(snap.clone()); snap
    }
    pub fn seal(&mut self){self.state=VmState::Sealed;}
    pub fn is_sealed(&self)->bool{self.state==VmState::Sealed}
    pub fn exec_allowed(&self,call:&str)->BkgResult<()>{
        if self.is_sealed(){return Err(BkgError::Internal(VmError::Sealed.to_string()));}
        if !self.filter.is_allowed(call){return Err(BkgError::Internal(VmError::SyscallDenied(call.into()).to_string()));}
        Ok(())
    }
}
#[cfg(test)]
mod tests{use super::*;
    use crate::mount::MountPolicy;
    #[test] fn create_and_snapshot(){
        let mut vm=SandboxVm::new(ResourceLimits::default());
        vm.add_mount(VfsMount::read_only("/host","/workspace"));
        vm.set_env("BKG_TASK","T-1");
        let snap=vm.snapshot("before-exec");
        assert!(!snap.checksum.is_empty());
        assert_eq!(vm.snapshots.len(),1);
    }
    #[test] fn seal_blocks_exec(){
        let mut vm=SandboxVm::new(ResourceLimits::strict());
        vm.seal();
        assert!(vm.exec_allowed("read").is_err());
    }
    #[test] fn allowed_syscall(){
        let vm=SandboxVm::new(ResourceLimits::default());
        assert!(vm.exec_allowed("read").is_ok());
        assert!(vm.exec_allowed("fork").is_err());
    }
}
