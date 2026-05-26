use serde::{Deserialize,Serialize};
use bkg_core::{BkgError,BkgResult};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RenderOutput{pub backend_name:String,pub width:u32,pub height:u32,pub data:Vec<u8>,pub format:String}
impl RenderOutput{pub fn new(name:&str,w:u32,h:u32,data:Vec<u8>,fmt:&str)->Self{Self{backend_name:name.into(),width:w,height:h,data,format:fmt.into()}}}

pub trait RenderBackend:Send+Sync{
    fn name(&self)->&str;
    fn render(&mut self,bytecode:&bkg_compiler::bytecode::Bytecode,width:u32,height:u32)->BkgResult<RenderOutput>;
    fn is_headless(&self)->bool{false}
}
