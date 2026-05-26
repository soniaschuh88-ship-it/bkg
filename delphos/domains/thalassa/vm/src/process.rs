use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ProcessResult{
    pub exit_code:i32, pub stdout:String, pub stderr:String,
    pub duration_ms:u64, pub timed_out:bool,
}
impl ProcessResult{
    pub fn success(stdout:impl Into<String>,ms:u64)->Self{Self{exit_code:0,stdout:stdout.into(),stderr:String::new(),duration_ms:ms,timed_out:false}}
    pub fn failure(code:i32,stderr:impl Into<String>,ms:u64)->Self{Self{exit_code:code,stdout:String::new(),stderr:stderr.into(),duration_ms:ms,timed_out:false}}
    pub fn timeout()->Self{Self{exit_code:-1,stdout:String::new(),stderr:"timed out".into(),duration_ms:0,timed_out:true}}
    pub fn succeeded(&self)->bool{self.exit_code==0&&!self.timed_out}
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VmProcess{
    pub id:String, pub command:String, pub args:Vec<String>,
    pub env:std::collections::HashMap<String,String>,
    pub working_dir:String, pub started_at:DateTime<Utc>,
    pub result:Option<ProcessResult>,
}
impl VmProcess{
    pub fn new(command:impl Into<String>,args:Vec<String>)->Self{
        Self{id:uuid::Uuid::new_v4().to_string(),command:command.into(),args,env:Default::default(),working_dir:"/".into(),started_at:Utc::now(),result:None}
    }
    pub fn with_working_dir(mut self,d:impl Into<String>)->Self{self.working_dir=d.into();self}
    pub fn is_done(&self)->bool{self.result.is_some()}
}
