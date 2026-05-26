use serde::{Deserialize, Serialize};
use crate::phase::WorkflowPhase;
use crate::verdict::Verdict;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig { pub phase: WorkflowPhase, pub max_retries: u32, pub is_blocking: bool, pub fallback_model: Option<String> }
impl GateConfig { pub fn blocking(phase: WorkflowPhase) -> Self { Self { phase, max_retries: 3, is_blocking: true, fallback_model: None } } pub fn informational(phase: WorkflowPhase) -> Self { Self { phase, max_retries: 1, is_blocking: false, fallback_model: None } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGate { pub config: GateConfig, pub retry_count: u32, pub last_verdict: Option<Verdict> }
impl WorkflowGate {
    pub fn new(config: GateConfig) -> Self { Self { config, retry_count: 0, last_verdict: None } }
    pub fn record_verdict(&mut self, v: Verdict) { self.last_verdict = Some(v); if v != Verdict::Approve { self.retry_count += 1; } }
    pub fn is_exhausted(&self) -> bool { self.retry_count >= self.config.max_retries }
    pub fn is_passed(&self) -> bool { self.last_verdict == Some(Verdict::Approve) }
}