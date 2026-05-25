use std::collections::HashMap;
use bkg_core::{BkgError,BkgResult,TaskId};
use petgraph::{algo::toposort,graph::{DiGraph,NodeIndex}};
use crate::task::{Task,TaskStatus};
pub struct TaskGraph{graph:DiGraph<Task,()>,index:HashMap<TaskId,NodeIndex>}
impl TaskGraph{
    pub fn new()->Self{Self{graph:DiGraph::new(),index:HashMap::new()}}
    pub fn add_task(&mut self,t:Task)->BkgResult<TaskId>{let id=t.id;if self.index.contains_key(&id){return Err(BkgError::Internal(format!("task {id} exists")));}let nx=self.graph.add_node(t);self.index.insert(id,nx);Ok(id)}
    pub fn add_dependency(&mut self,task:TaskId,after:TaskId)->BkgResult<()>{
        let tnx=*self.index.get(&task).ok_or_else(||BkgError::Internal(format!("task {task}")))?;
        let anx=*self.index.get(&after).ok_or_else(||BkgError::Internal(format!("task {after}")))?;
        self.graph.add_edge(anx,tnx,());
        if toposort(&self.graph,None).is_err(){if let Some(e)=self.graph.find_edge(anx,tnx){self.graph.remove_edge(e);}return Err(BkgError::Internal("cycle detected".into()));}
        Ok(())
    }
    pub fn complete_task(&mut self,id:TaskId)->BkgResult<Vec<TaskId>>{
        let nx=*self.index.get(&id).ok_or_else(||BkgError::Internal(format!("task {id}")))?;
        self.graph[nx].status=TaskStatus::Completed;self.graph[nx].completed_at=Some(chrono::Utc::now());
        let deps:Vec<NodeIndex>=self.graph.neighbors_directed(nx,petgraph::Direction::Outgoing).collect();
        let mut ready=Vec::new();
        for dnx in deps{if self.prereqs_met(dnx)&&self.graph[dnx].status==TaskStatus::Pending{self.graph[dnx].status=TaskStatus::Ready;ready.push(self.graph[dnx].id);}}
        Ok(ready)
    }
    pub fn fail_task(&mut self,id:TaskId)->BkgResult<()>{let nx=*self.index.get(&id).ok_or_else(||BkgError::Internal(format!("task {id}")))?;self.graph[nx].status=TaskStatus::Failed;self.graph[nx].completed_at=Some(chrono::Utc::now());Ok(())}
    pub fn resolve_initial_ready(&mut self)->Vec<TaskId>{let cands:Vec<NodeIndex>=self.graph.node_indices().collect();let mut ready=Vec::new();for nx in cands{if self.graph[nx].status==TaskStatus::Pending&&self.prereqs_met(nx){self.graph[nx].status=TaskStatus::Ready;ready.push(self.graph[nx].id);}}ready}
    fn prereqs_met(&self,nx:NodeIndex)->bool{self.graph.neighbors_directed(nx,petgraph::Direction::Incoming).all(|p|self.graph[p].is_terminal())}
    pub fn ready_tasks(&self)->Vec<&Task>{let mut v:Vec<_>=self.graph.node_indices().map(|nx|&self.graph[nx]).filter(|t|t.status==TaskStatus::Ready).collect();v.sort_by_key(|t|std::cmp::Reverse(t.priority));v}
    pub fn pending_tasks(&self)->Vec<&Task>{self.graph.node_indices().map(|nx|&self.graph[nx]).filter(|t|t.status==TaskStatus::Pending).collect()}
    pub fn task(&self,id:&TaskId)->Option<&Task>{self.index.get(id).map(|&nx|&self.graph[nx])}
    pub fn task_mut(&mut self,id:TaskId)->Option<&mut Task>{self.index.get(&id).copied().map(|nx|&mut self.graph[nx])}
    pub fn task_count(&self)->usize{self.graph.node_count()}
}
impl Default for TaskGraph{fn default()->Self{Self::new()}}
#[cfg(test)]mod tests{use super::*;use bkg_core::ExecutionSeed;use bkg_runtime::TaskPayload;
    fn t(l:&str)->Task{Task::new(l,TaskPayload::new(l,serde_json::json!({"action":"noop"})).with_seed(ExecutionSeed::random()),crate::task::TaskPriority::Normal)}
    #[test]fn add(){let mut g=TaskGraph::new();g.add_task(t("t1")).unwrap();g.add_task(t("t2")).unwrap();assert_eq!(g.resolve_initial_ready().len(),2);}
    #[test]fn dep(){let mut g=TaskGraph::new();let i1=g.add_task(t("t1")).unwrap();let i2=g.add_task(t("t2")).unwrap();g.add_dependency(i2,i1).unwrap();g.resolve_initial_ready();assert_eq!(g.ready_tasks()[0].id,i1);}
    #[test]fn complete_unblocks(){let mut g=TaskGraph::new();let i1=g.add_task(t("t1")).unwrap();let i2=g.add_task(t("t2")).unwrap();g.add_dependency(i2,i1).unwrap();g.resolve_initial_ready();let r=g.complete_task(i1).unwrap();assert!(r.contains(&i2));}
    #[test]fn cycle(){let mut g=TaskGraph::new();let i1=g.add_task(t("t1")).unwrap();let i2=g.add_task(t("t2")).unwrap();g.add_dependency(i2,i1).unwrap();assert!(g.add_dependency(i1,i2).is_err());}
}
