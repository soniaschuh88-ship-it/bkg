//! ExecutionGraph: formal workflow graph with loops, retries, waves, conditionals.
use serde::{Deserialize, Serialize};
use crate::phase::WorkflowPhase;
use crate::verdict::Verdict;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum EdgeKind { Approve, Revise, Rethink, Timeout, Unconditional }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode { pub phase: WorkflowPhase, pub is_terminal: bool, pub wave_index: u32 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge { pub from: WorkflowPhase, pub to: Option<WorkflowPhase>, pub condition: EdgeKind }
/// Formal workflow execution graph. Not just a DAG — supports loops (retries) and parallel waves.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> }
impl ExecutionGraph {
    pub fn default_plan_execute() -> Self {
        Self {
            nodes: vec![
                GraphNode { phase: WorkflowPhase::Plan, is_terminal: false, wave_index: 0 },
                GraphNode { phase: WorkflowPhase::PlanReview, is_terminal: false, wave_index: 0 },
                GraphNode { phase: WorkflowPhase::Execute, is_terminal: false, wave_index: 0 },
                GraphNode { phase: WorkflowPhase::ExecuteReview, is_terminal: true, wave_index: 0 },
            ],
            edges: vec![
                GraphEdge { from: WorkflowPhase::Plan, to: Some(WorkflowPhase::PlanReview), condition: EdgeKind::Unconditional },
                GraphEdge { from: WorkflowPhase::PlanReview, to: Some(WorkflowPhase::Execute), condition: EdgeKind::Approve },
                GraphEdge { from: WorkflowPhase::PlanReview, to: Some(WorkflowPhase::Plan), condition: EdgeKind::Revise },
                GraphEdge { from: WorkflowPhase::PlanReview, to: None, condition: EdgeKind::Rethink },
                GraphEdge { from: WorkflowPhase::Execute, to: Some(WorkflowPhase::ExecuteReview), condition: EdgeKind::Unconditional },
                GraphEdge { from: WorkflowPhase::ExecuteReview, to: None, condition: EdgeKind::Approve },
                GraphEdge { from: WorkflowPhase::ExecuteReview, to: Some(WorkflowPhase::Execute), condition: EdgeKind::Revise },
            ],
        }
    }
    pub fn next_phase(&self, current: WorkflowPhase, verdict: Verdict) -> Option<WorkflowPhase> {
        let condition = match verdict { Verdict::Approve=>EdgeKind::Approve, Verdict::Revise=>EdgeKind::Revise, Verdict::Rethink=>EdgeKind::Rethink, Verdict::Unavailable=>EdgeKind::Revise };
        self.edges.iter().find(|e| e.from == current && (e.condition == condition || e.condition == EdgeKind::Unconditional)).and_then(|e| e.to)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn default_graph_approve() { let g=ExecutionGraph::default_plan_execute(); assert_eq!(g.next_phase(WorkflowPhase::PlanReview,Verdict::Approve),Some(WorkflowPhase::Execute)); }
    #[test] fn revise_loops_back() { let g=ExecutionGraph::default_plan_execute(); assert_eq!(g.next_phase(WorkflowPhase::PlanReview,Verdict::Revise),Some(WorkflowPhase::Plan)); }
    #[test] fn rethink_terminates() { let g=ExecutionGraph::default_plan_execute(); assert_eq!(g.next_phase(WorkflowPhase::PlanReview,Verdict::Rethink),None); }
    #[test] fn execute_review_approve_done() { let g=ExecutionGraph::default_plan_execute(); assert_eq!(g.next_phase(WorkflowPhase::ExecuteReview,Verdict::Approve),None); }
}