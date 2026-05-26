use serde::{Deserialize,Serialize};
use std::path::PathBuf;
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum MountPolicy{ReadOnly,ReadWrite,Hidden}
impl MountPolicy{pub fn allows_write(self)->bool{self==Self::ReadWrite}}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VfsMount{pub host_path:PathBuf,pub guest_path:PathBuf,pub policy:MountPolicy}
impl VfsMount{
    pub fn new(host:impl Into<std::path::PathBuf>,guest:impl Into<std::path::PathBuf>,policy:MountPolicy)->Self{Self{host_path:host.into(),guest_path:guest.into(),policy}}
    pub fn read_only(host:impl Into<std::path::PathBuf>,guest:impl Into<std::path::PathBuf>)->Self{Self::new(host,guest,MountPolicy::ReadOnly)}
    pub fn read_write(host:impl Into<std::path::PathBuf>,guest:impl Into<std::path::PathBuf>)->Self{Self::new(host,guest,MountPolicy::ReadWrite)}
    pub fn can_write(&self,path:&std::path::Path)->bool{path.starts_with(&self.guest_path)&&self.policy.allows_write()}
}
