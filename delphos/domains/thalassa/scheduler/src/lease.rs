use serde::{Deserialize,Serialize};use chrono::{DateTime,Duration,Utc};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TaskLease{pub task_id:String,pub holder:String,pub epoch:u64,pub expires_at:DateTime<Utc>}
impl TaskLease{
    pub fn new(task_id:impl Into<String>,holder:impl Into<String>,epoch:u64,ttl_secs:i64)->Self{Self{task_id:task_id.into(),holder:holder.into(),epoch,expires_at:Utc::now()+Duration::seconds(ttl_secs)}}
    pub fn is_expired(&self)->bool{Utc::now()>self.expires_at}
    pub fn is_held_by(&self,holder:&str)->bool{self.holder==holder&&!self.is_expired()}
}