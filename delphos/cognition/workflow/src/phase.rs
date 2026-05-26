use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum WorkflowPhase { Plan, PlanReview, Execute, ExecuteReview }
impl WorkflowPhase {
    pub fn as_str(self) -> &'static str { match self { Self::Plan=>"plan", Self::PlanReview=>"plan_review", Self::Execute=>"execute", Self::ExecuteReview=>"execute_review" } }
    pub fn is_review(self) -> bool { matches!(self, Self::PlanReview | Self::ExecuteReview) }
    pub fn next(self) -> Option<Self> { match self { Self::Plan=>Some(Self::PlanReview), Self::PlanReview=>Some(Self::Execute), Self::Execute=>Some(Self::ExecuteReview), Self::ExecuteReview=>None } }
}
impl std::fmt::Display for WorkflowPhase { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_str()) } }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
pub enum WorkflowPhaseStatus { #[default] Idle, Running, AwaitingReview, Blocked, Complete, Failed }