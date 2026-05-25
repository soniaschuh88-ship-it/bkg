use std::collections::HashMap;
use bkg_capsule::CapsuleManager;
use bkg_core::{AgentId,BkgError,BkgResult,Capability,LogicalTimestamp,RealmId,SessionId};
use bkg_crypto::signing::KeyPair;
use bkg_swd::{ReadOp,SwdEngine,WriteOp};
use crate::{agent::{Agent,AgentStatus},execution::{ExecutionOutcome,ExecutionResult,TaskPayload}};

pub struct AgentRuntime{agents:HashMap<AgentId,Agent>,cm:CapsuleManager,swd:SwdEngine,tick:u64}
impl AgentRuntime{
    pub fn new()->Self{Self{agents:HashMap::new(),cm:CapsuleManager::new(),swd:SwdEngine::new(),tick:0}}
    pub fn spawn(&mut self,name:impl Into<String>,caps:Vec<Capability>,pk:Option<bkg_crypto::PublicKey>)->BkgResult<AgentId>{
        let c=self.cm.create(RealmId::Causa,None,serde_json::json!({"state":"idle","tasks":0}))?;
        let a=Agent::new(name,caps,c.capsule_id,pk);let id=a.id;self.agents.insert(id,a);Ok(id)
    }
    pub fn terminate(&mut self,id:AgentId)->BkgResult<()>{
        let a=self.agents.get_mut(&id).ok_or_else(||BkgError::Internal(format!("agent {id}")))?;
        if a.status==AgentStatus::Running{return Err(BkgError::Internal("running".into()));}
        a.status=AgentStatus::Terminated;self.cm.retire(a.capsule_id)?;Ok(())
    }
    pub fn execute(&mut self,agent_id:AgentId,payload:TaskPayload,kp:Option<&KeyPair>)->BkgResult<ExecutionResult>{
        let cid={let a=self.agents.get_mut(&agent_id).ok_or_else(||BkgError::Internal(format!("agent {agent_id}")))?;if!a.is_available(){return Err(BkgError::Internal("not available".into()));}a.status=AgentStatus::Running;a.capsule_id};
        let sid=SessionId::new();
        let ib=serde_json::to_vec(&payload.input).unwrap_or_default();
        self.swd.init(sid,RealmId::Telum,payload.seed,&ib)?;
        self.swd.capture_realm_enter(RealmId::Telum,LogicalTimestamp(self.tick),&payload.label)?;
        let t0=self.tick;self.tick+=1;
        let(outcome,output)=self.sandbox(&payload);
        self.swd.capture_write(WriteOp::new(LogicalTimestamp(self.tick),RealmId::Causa,"capsule.update"))?;
        self.swd.capture_read(ReadOp::new(LogicalTimestamp(self.tick),RealmId::Telum,"payload.read",&payload.label))?;
        self.swd.capture_realm_exit(LogicalTimestamp(self.tick))?;
        self.swd.capture_event(bkg_core::EventId::new())?;
        self.swd.add_budget(1)?;
        let swd=self.swd.commit(kp)?;
        let ns=serde_json::json!({"state":format!("{:?}",outcome).to_lowercase(),"last":payload.label,"tasks":self.agents.get(&agent_id).map(|a|a.task_count+1).unwrap_or(1)});
        self.cm.update(cid,ns,Some(sid))?;
        if let Some(a)=self.agents.get_mut(&agent_id){a.status=AgentStatus::Idle;a.task_count+=1;a.active_session=None;}
        let ticks=self.tick-t0;self.tick+=1;
        Ok(match outcome{ExecutionOutcome::Success=>ExecutionResult::success(agent_id,swd.session_id,output,ticks),_=>ExecutionResult::failure(agent_id,swd.session_id,"task failed")})
    }
    fn sandbox(&self,p:&TaskPayload)->(ExecutionOutcome,serde_json::Value){
        match p.input.get("action").and_then(|v|v.as_str()).unwrap_or("echo"){
            "echo"=>(ExecutionOutcome::Success,serde_json::json!({"echoed":p.input.get("data").cloned().unwrap_or(serde_json::Value::Null)})),
            "fail"=>(ExecutionOutcome::Failure,serde_json::Value::Null),
            other=>(ExecutionOutcome::Success,serde_json::json!({"action":other})),
        }
    }
    pub fn agent(&self,id:&AgentId)->Option<&Agent>{self.agents.get(id)}
    pub fn active_agents(&self)->Vec<&Agent>{self.agents.values().filter(|a|a.status!=AgentStatus::Terminated).collect()}
    pub fn agent_count(&self)->usize{self.agents.len()}
}
impl Default for AgentRuntime{fn default()->Self{Self::new()}}

#[cfg(test)]mod tests{use super::*;use bkg_core::{Capability,ExecutionSeed};
    fn rt()->(AgentRuntime,AgentId){let mut r=AgentRuntime::new();let id=r.spawn("a",vec![Capability::RuntimeExecute],None).unwrap();(r,id)}
    #[test]fn spawn(){let(r,id)=rt();assert!(r.agent(&id).unwrap().is_available());}
    #[test]fn echo(){let(mut r,id)=rt();let res=r.execute(id,TaskPayload::new("t",serde_json::json!({"action":"echo","data":"hi"})).with_seed(ExecutionSeed::from_bytes([1u8;32])),None).unwrap();assert!(res.is_success());assert_eq!(res.output["echoed"],"hi");}
    #[test]fn fail(){let(mut r,id)=rt();assert!(!r.execute(id,TaskPayload::new("t",serde_json::json!({"action":"fail"})),None).unwrap().is_success());}
    #[test]fn task_count(){let(mut r,id)=rt();for _ in 0..3{r.execute(id,TaskPayload::new("t",serde_json::json!({"action":"noop"})),None).unwrap();}assert_eq!(r.agent(&id).unwrap().task_count,3);}
    #[test]fn terminate(){let(mut r,id)=rt();r.terminate(id).unwrap();assert_eq!(r.agent(&id).unwrap().status,AgentStatus::Terminated);}
}
