use serde::{Deserialize,Serialize};use bkg_core::{BkgError,BkgResult};
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]#[serde(rename_all="snake_case")]
pub enum PermissionMode{ReadOnly,WorkspaceWrite,DangerFullAccess}
impl PermissionMode{pub fn as_str(self)->&'static str{match self{Self::ReadOnly=>"read-only",Self::WorkspaceWrite=>"workspace-write",Self::DangerFullAccess=>"danger-full-access"}}pub fn allows_write(self)->bool{self>=Self::WorkspaceWrite}pub fn allows_full_access(self)->bool{self==Self::DangerFullAccess}}
impl std::fmt::Display for PermissionMode{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(self.as_str())}}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]#[serde(rename_all="snake_case")]
pub enum EnforcementResult{Allow,Deny{reason:String},Prompt{message:String}}
impl EnforcementResult{pub fn is_allowed(&self)->bool{matches!(self,Self::Allow)}pub fn is_denied(&self)->bool{matches!(self,Self::Deny{..})}pub fn reason(&self)->Option<&str>{match self{Self::Deny{reason}|Self::Prompt{message:reason}=>Some(reason),Self::Allow=>None}}}
#[derive(Debug,Clone)]pub struct PermissionRequest{pub tool_name:String,pub input:String,pub mode:PermissionMode,pub path:Option<String>}
impl PermissionRequest{pub fn new(t:impl Into<String>,i:impl Into<String>,m:PermissionMode)->Self{let i=i.into();let p=serde_json::from_str::<serde_json::Value>(&i).ok().and_then(|v|v.get("path").and_then(|p|p.as_str()).map(String::from));Self{tool_name:t.into(),input:i,mode:m,path:p}}}
#[derive(Debug,Default)]pub struct PermissionEnforcer;
impl PermissionEnforcer{
    pub fn new()->Self{Self}
    pub fn check(&self,req:&PermissionRequest)->EnforcementResult{match req.mode{PermissionMode::ReadOnly=>{if self.rw(&req.tool_name){return EnforcementResult::Deny{reason:format!("'{}' requires workspace-write; current mode is read-only",req.tool_name)};}}PermissionMode::WorkspaceWrite=>{if self.full(&req.tool_name){return EnforcementResult::Prompt{message:format!("'{}' requires danger-full-access",req.tool_name)};}}PermissionMode::DangerFullAccess=>{}}EnforcementResult::Allow}
    pub fn check_all(&self,reqs:&[PermissionRequest])->BkgResult<()>{for r in reqs{if let EnforcementResult::Deny{reason}=self.check(r){return Err(BkgError::MissingCapability(format!("denied '{}': {reason}",r.tool_name)));}}Ok(())}
    fn rw(&self,n:&str)->bool{matches!(n,"bash"|"write_file"|"edit_file"|"delete_file"|"git_commit"|"git_push")}
    fn full(&self,n:&str)->bool{matches!(n,"dangerously_allow_any"|"network_unrestricted")}
}
#[cfg(test)]mod tests{use super::*;
    fn req(t:&str,m:PermissionMode)->PermissionRequest{PermissionRequest::new(t,"{}",m)}
    #[test]fn ro_blocks(){assert!(PermissionEnforcer::new().check(&req("bash",PermissionMode::ReadOnly)).is_denied());}
    #[test]fn rw_allows(){assert!(PermissionEnforcer::new().check(&req("bash",PermissionMode::WorkspaceWrite)).is_allowed());}
    #[test]fn ro_read(){assert!(PermissionEnforcer::new().check(&req("read_file",PermissionMode::ReadOnly)).is_allowed());}
    #[test]fn danger(){assert!(PermissionEnforcer::new().check(&req("bash",PermissionMode::DangerFullAccess)).is_allowed());}
    #[test]fn order(){assert!(PermissionMode::WorkspaceWrite>PermissionMode::ReadOnly);}
}
