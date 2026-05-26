use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CausalLink { pub cause_event_id: String, pub effect_entity_id: String, pub effect_type: String, pub strength: f64, pub created_at: DateTime<Utc> }
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct CausalChain { pub links: Vec<CausalLink> }
impl CausalChain {
    pub fn new() -> Self { Self::default() }
    pub fn record(&mut self, cause: impl Into<String>, entity: impl Into<String>, etype: impl Into<String>, strength: f64) {
        self.links.push(CausalLink { cause_event_id: cause.into(), effect_entity_id: entity.into(), effect_type: etype.into(), strength, created_at: Utc::now() });
    }
    pub fn causes_of(&self, id: &str) -> Vec<&CausalLink> { self.links.iter().filter(|l| l.effect_entity_id == id).collect() }
    pub fn depth(&self) -> usize { self.links.len() }
}
