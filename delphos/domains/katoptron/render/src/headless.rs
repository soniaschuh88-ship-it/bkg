use bkg_core::BkgResult;
use bkg_compiler::bytecode::Bytecode;
use crate::backend::{RenderBackend,RenderOutput};
pub struct HeadlessBackend{pub pixel_buf:Vec<u8>}
impl HeadlessBackend{pub fn new()->Self{Self{pixel_buf:vec![]}}}
impl RenderBackend for HeadlessBackend{
    fn name(&self)->&str{"headless"}
    fn is_headless(&self)->bool{true}
    fn render(&mut self,_bc:&Bytecode,width:u32,height:u32)->BkgResult<RenderOutput>{
        let size=(width*height*4)as usize;
        self.pixel_buf=vec![0u8;size];
        Ok(RenderOutput::new("headless",width,height,self.pixel_buf.clone(),"rgba"))
    }
}
#[cfg(test)]
mod tests{use super::*;use bkg_compiler::bytecode::Bytecode;
    #[test] fn renders(){let mut b=HeadlessBackend::new();let bc=Bytecode::new("t",1);let out=b.render(&bc,80,25).unwrap();assert_eq!(out.data.len(),(80*25*4)as usize);}
}
