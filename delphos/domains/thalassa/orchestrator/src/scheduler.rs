use bkg_core::{AgentId,BkgResult,TaskId};
use bkg_runtime::{AgentRuntime,ExecutionResult};
use crate::{bus::EventBus,graph::TaskGraph,task::{Task,TaskStatus}};
pub struct Scheduler{pub task_graph:TaskGraph,pub event_bus:EventBus,pub runtime:AgentRuntime}
impl Scheduler{
    pub fn new(rt:AgentRuntime)->Self{Self{task_graph:TaskGraph::new(),event_bus:EventBus::new(),runtime:rt}}
    pub fn add_task(&mut self,t:Task)->BkgResult<TaskId>{self.task_graph.add_task(t)}
    pub fn add_dependency(&mut self,t:TaskId,after:TaskId)->BkgResult<()>{self.task_graph.add_dependency(t,after)}
    pub fn prepare(&mut self){self.task_graph.resolve_initial_ready();}
    pub fn tick(&mut self,agent:AgentId)->BkgResult<Vec<(TaskId,ExecutionResult)>>{
        let ids:Vec<TaskId>=self.task_graph.ready_tasks().iter().map(|t|t.id).collect();
        let mut results=Vec::new();
        for id in ids{
            if self.runtime.agent(&agent).map(|a|!a.is_available()).unwrap_or(true){break;}
            let payload=self.task_graph.task(&id).unwrap().payload.clone();
            if let Some(t)=self.task_graph.task_mut(id){t.status=TaskStatus::Running;t.assigned_agent=Some(agent);}
            match self.runtime.execute(agent,payload,None){
                Ok(r)=>{self.event_bus.publish("task.completed",serde_json::json!({"id":id.to_string()})).ok();self.task_graph.complete_task(id)?;results.push((id,r));}
                Err(e)=>{self.event_bus.publish("task.failed",serde_json::json!({"id":id.to_string(),"err":e.to_string()})).ok();self.task_graph.fail_task(id)?;}
            }
        }
        Ok(results)
    }
    pub fn is_complete(&self)->bool{self.task_graph.ready_tasks().is_empty()&&self.task_graph.pending_tasks().is_empty()}
}
#[cfg(test)]mod tests{use super::*;use bkg_core::{Capability,ExecutionSeed};use bkg_runtime::TaskPayload;use crate::task::{Task,TaskPriority};
    fn make()->(Scheduler,AgentId){let mut rt=AgentRuntime::new();let id=rt.spawn("a",vec![Capability::RuntimeExecute],None).unwrap();(Scheduler::new(rt),id)}
    fn task(l:&str)->Task{Task::new(l,TaskPayload::new(l,serde_json::json!({"action":"noop"})).with_seed(ExecutionSeed::random()),TaskPriority::Normal)}
    #[test]fn single(){let(mut s,a)=make();s.add_task(task("t1")).unwrap();s.prepare();let r=s.tick(a).unwrap();assert_eq!(r.len(),1);assert!(r[0].1.is_success());}
    #[test]fn dep_order(){let(mut s,a)=make();let i1=s.add_task(task("t1")).unwrap();let i2=s.add_task(task("t2")).unwrap();s.add_dependency(i2,i1).unwrap();s.prepare();let r1=s.tick(a).unwrap();assert_eq!(r1[0].0,i1);let r2=s.tick(a).unwrap();assert_eq!(r2[0].0,i2);}
}
