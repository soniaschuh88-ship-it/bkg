use serde::{Deserialize,Serialize};
use bkg_core::{LogicalTimestamp,RealmId};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RealmTrace{pub realm:RealmId,pub entered_at:LogicalTimestamp,#[serde(default,skip_serializing_if="Option::is_none")]pub exited_at:Option<LogicalTimestamp>,pub label:String,pub events_appended:u32,pub contracts_issued:u32}
impl RealmTrace{
    pub fn enter(r:RealmId,t:LogicalTimestamp,l:impl Into<String>)->Self{Self{realm:r,entered_at:t,exited_at:None,label:l.into(),events_appended:0,contracts_issued:0}}
    pub fn exit(&mut self,t:LogicalTimestamp){self.exited_at=Some(t);}
    pub fn is_active(&self)->bool{self.exited_at.is_none()}
    pub fn duration_ticks(&self)->Option<u64>{self.exited_at.map(|e|e.as_u64().saturating_sub(self.entered_at.as_u64()))}
}
