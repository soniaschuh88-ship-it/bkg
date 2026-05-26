use serde::{Deserialize,Serialize};
/// Virtualized syscall record — replay-safe I/O tracking.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SyscallRecord{pub id:u64,pub call:String,pub args:Vec<String>,pub result:String,pub allowed:bool}
impl SyscallRecord{
    pub fn new(id:u64,call:impl Into<String>,args:Vec<String>,result:impl Into<String>,allowed:bool)->Self{Self{id,call:call.into(),args,result:result.into(),allowed}}
    pub fn allowed(id:u64,call:impl Into<String>,result:impl Into<String>)->Self{Self::new(id,call,vec![],result,true)}
    pub fn denied(id:u64,call:impl Into<String>)->Self{Self::new(id,call,vec![],"EPERM",false)}
}
/// Decides whether a syscall is permitted given the current sandbox policy.
pub struct SyscallFilter{allowed_calls:Vec<String>}
impl SyscallFilter{
    pub fn new(allowed:Vec<String>)->Self{Self{allowed_calls:allowed}}
    pub fn minimal()->Self{Self::new(vec!["read".into(),"write".into(),"exit".into()])}
    pub fn permissive()->Self{Self::new(vec!["*".into()])}
    pub fn is_allowed(&self,call:&str)->bool{self.allowed_calls.iter().any(|a|a=="*"||a==call)}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn minimal(){let f=SyscallFilter::minimal();assert!(f.is_allowed("read"));assert!(!f.is_allowed("fork"));}
    #[test] fn permissive(){assert!(SyscallFilter::permissive().is_allowed("anything"));}
}
