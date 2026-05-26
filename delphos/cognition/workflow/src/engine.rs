use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bkg_core::BkgError;
use crate::{graph::ExecutionGraph, phase::{WorkflowPhase, WorkflowPhaseStatus}, verdict::{Verdict, VerdictRecord}};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState { pub task_id: String, pub current_phase: WorkflowPhase, pub phase_status: WorkflowPhaseStatus, pub verdicts: Vec<VerdictRecord>, pub retry_count: u32, pub max_retries: u32, pub graph: ExecutionGraph, pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc> }
impl WorkflowState {
    pub fn new(task_id: impl Into<String>) -> Self { let now=Utc::now(); Self { task_id: task_id.into(), current_phase: WorkflowPhase::Plan, phase_status: WorkflowPhaseStatus::Idle, verdicts: vec![], retry_count: 0, max_retries: 3, graph: ExecutionGraph::default_plan_execute(), created_at: now, updated_at: now } }
    pub fn is_complete(&self) -> bool { matches!(self.phase_status, WorkflowPhaseStatus::Complete | WorkflowPhaseStatus::Failed) }
    pub fn advance(&mut self, verdict: Verdict, feedback: impl Into<String>, model: Option<String>) -> bkg_core::BkgResult<Option<WorkflowPhase>> {
        let rec = VerdictRecord::new(verdict, feedback, model);
        self.verdicts.push(rec);
        self.updated_at = Utc::now();
        match self.graph.next_phase(self.current_phase, verdict) {
            Some(next) => { self.current_phase = next; self.phase_status = WorkflowPhaseStatus::Idle; Ok(Some(next)) }
            None => {
                match verdict {
                    Verdict::Approve => { self.phase_status = WorkflowPhaseStatus::Complete; }
                    Verdict::Rethink => { self.phase_status = WorkflowPhaseStatus::Failed; }
                    _ => { self.retry_count += 1; if self.retry_count >= self.max_retries { self.phase_status = WorkflowPhaseStatus::Failed; } }
                }
                Ok(None)
            }
        }
    }
}
#[derive(Debug, Default)]
pub struct WorkflowEngine { states: HashMap<String, WorkflowState> }
impl WorkflowEngine {
    pub fn new() -> Self { Self::default() }
    pub fn start(&mut self, task_id: impl Into<String>) -> &WorkflowState { let s=WorkflowState::new(task_id.into()); let id=s.task_id.clone(); self.states.insert(id.clone(), s); self.states.get(&id).unwrap() }
    pub fn state(&self, task_id: &str) -> Option<&WorkflowState> { self.states.get(task_id) }
    pub fn advance(&mut self, task_id: &str, verdict: Verdict, feedback: impl Into<String>, model: Option<String>) -> bkg_core::BkgResult<Option<WorkflowPhase>> { let s=self.states.get_mut(task_id).ok_or_else(||BkgError::Internal(format!("task {task_id} not in workflow")))?; s.advance(verdict, feedback, model) }
    pub fn count(&self) -> usize { self.states.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn full_approve_cycle() {
        let mut e=WorkflowEngine::new(); e.start("T-1");
        // Plan → PlanReview (unconditional)
        e.advance("T-1",Verdict::Approve,"ok",None).unwrap();
        assert_eq!(e.state("T-1").unwrap().current_phase,WorkflowPhase::PlanReview);
        // PlanReview → Execute (APPROVE)
        e.advance("T-1",Verdict::Approve,"approved",None).unwrap();
        assert_eq!(e.state("T-1").unwrap().current_phase,WorkflowPhase::Execute);
        // Execute → ExecuteReview (unconditional)
        e.advance("T-1",Verdict::Approve,"ok",None).unwrap();
        assert_eq!(e.state("T-1").unwrap().current_phase,WorkflowPhase::ExecuteReview);
        // ExecuteReview → Complete (APPROVE returns None)
        e.advance("T-1",Verdict::Approve,"done",None).unwrap();
        assert!(e.state("T-1").unwrap().is_complete());
    }
    #[test] fn revise_loops_back() {
        let mut e=WorkflowEngine::new(); e.start("T-2");
        // Plan → PlanReview (unconditional)
        e.advance("T-2",Verdict::Approve,"ok",None).unwrap();
        assert_eq!(e.state("T-2").unwrap().current_phase,WorkflowPhase::PlanReview);
        // PlanReview → Plan (REVISE loops back)
        e.advance("T-2",Verdict::Revise,"needs work",None).unwrap();
        assert_eq!(e.state("T-2").unwrap().current_phase,WorkflowPhase::Plan);
    }
    #[test] fn rethink_fails() {
        let mut e=WorkflowEngine::new(); e.start("T-3");
        // Plan → PlanReview
        e.advance("T-3",Verdict::Approve,"ok",None).unwrap();
        // PlanReview → None (RETHINK terminates)
        e.advance("T-3",Verdict::Rethink,"no",None).unwrap();
        assert_eq!(e.state("T-3").unwrap().phase_status,WorkflowPhaseStatus::Failed);
    }
}