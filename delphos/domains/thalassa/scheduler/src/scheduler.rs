use std::collections::{HashMap,HashSet};
use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
use crate::priority::SchedulerPriority;
#[derive(Debug,thiserror::Error)]
pub enum SchedulerError{#[error("task {0} not found")]NotFound(String),#[error("overlap conflict: tasks {a} and {b} share files")]OverlapConflict{a:String,b:String}}
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct ScheduledTask{pub task_id:String,pub priority:SchedulerPriority,pub created_at:DateTime<Utc>,pub dependencies:HashSet<String>,pub file_scope:Vec<String>}
impl ScheduledTask{pub fn new(task_id:impl Into<String>,priority:SchedulerPriority)->Self{Self{task_id:task_id.into(),priority,created_at:Utc::now(),dependencies:HashSet::new(),file_scope:vec![]}}}
impl PartialOrd for ScheduledTask{fn partial_cmp(&self,o:&Self)->Option<std::cmp::Ordering>{Some(self.cmp(o))}}
impl Ord for ScheduledTask{fn cmp(&self,o:&Self)->std::cmp::Ordering{self.priority.cmp(&o.priority).then(self.created_at.cmp(&o.created_at).reverse())}}
#[derive(Debug,Default)]
pub struct TaskScheduler{queue:Vec<ScheduledTask>,active:HashMap<String,ScheduledTask>,done:HashSet<String>}
impl TaskScheduler{
    pub fn new()->Self{Self::default()}
    pub fn enqueue(&mut self,t:ScheduledTask){self.queue.push(t);self.queue.sort_by(|a,b|b.priority.cmp(&a.priority).then(a.created_at.cmp(&b.created_at)));}
    pub fn next_ready(&mut self)->Option<ScheduledTask>{
        let done=&self.done;
        let pos=self.queue.iter().position(|t|t.dependencies.iter().all(|d|done.contains(d)));
        pos.map(|i|{let t=self.queue.remove(i);self.active.insert(t.task_id.clone(),t.clone());t})
    }
    pub fn complete(&mut self,task_id:&str){self.active.remove(task_id);self.done.insert(task_id.to_string());}
    pub fn queue_len(&self)->usize{self.queue.len()}
    pub fn active_count(&self)->usize{self.active.len()}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn priority_ordering(){
        let mut s=TaskScheduler::new();
        s.enqueue(ScheduledTask::new("T-low",SchedulerPriority::Low));
        s.enqueue(ScheduledTask::new("T-urgent",SchedulerPriority::Urgent));
        let first=s.next_ready().unwrap();
        assert_eq!(first.task_id,"T-urgent");
    }
    #[test] fn dependency_blocks(){
        let mut s=TaskScheduler::new();
        let mut t2=ScheduledTask::new("T-2",SchedulerPriority::Normal);
        t2.dependencies.insert("T-1".into());
        s.enqueue(t2);
        assert!(s.next_ready().is_none()); // T-1 not done
        s.done.insert("T-1".into());
        assert!(s.next_ready().is_some());
    }
    #[test] fn complete_unblocks(){
        let mut s=TaskScheduler::new();
        s.enqueue(ScheduledTask::new("T-1",SchedulerPriority::Normal));
        let t1=s.next_ready().unwrap();
        s.complete(&t1.task_id);
        assert_eq!(s.active_count(),0);
        assert!(s.done.contains("T-1"));
    }
}