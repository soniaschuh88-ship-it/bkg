use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
use crate::bytecode::Bytecode;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UiFrame{pub tick:u64,pub bytecode:Bytecode,pub width:u32,pub height:u32,pub rendered_at:DateTime<Utc>}
impl UiFrame{pub fn new(tick:u64,bc:Bytecode,w:u32,h:u32)->Self{Self{tick,bytecode:bc,width:w,height:h,rendered_at:Utc::now()}}}
