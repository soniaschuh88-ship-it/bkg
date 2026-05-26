use serde::{Deserialize,Serialize};
/// Portable render instruction — backend-agnostic.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum BytecodeOp{
    BeginPanel{id:String},EndPanel,
    Text{id:String,value:String,style:String},
    Badge{id:String,label:String,color:String},
    Button{id:String,label:String,action:String},
    BeginRow{id:String},BeginColumn{id:String},End,
    Spacer{size:u32},
}
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct Bytecode{pub ops:Vec<BytecodeOp>,pub realm_id:String,pub state_version:u64}
impl Bytecode{
    pub fn new(realm:impl Into<String>,version:u64)->Self{Self{ops:vec![],realm_id:realm.into(),state_version:version}}
    pub fn push(&mut self,op:BytecodeOp){self.ops.push(op);}
    pub fn len(&self)->usize{self.ops.len()}
    pub fn is_empty(&self)->bool{self.ops.is_empty()}
}
