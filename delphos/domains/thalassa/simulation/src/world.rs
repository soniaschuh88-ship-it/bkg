use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, Default)]
pub struct SimTick(pub u64);
impl SimTick { pub fn advance(&mut self) -> u64 { self.0 += 1; self.0 } }
#[derive(Debug, Default)]
pub struct SimWorld { pub tick: SimTick, pub entities: BTreeMap<String, serde_json::Value>, pub events: Vec<serde_json::Value>, pub log: Vec<String> }
impl SimWorld {
    pub fn new() -> Self { Self::default() }
    pub fn set_entity(&mut self, key: impl Into<String>, val: serde_json::Value) { self.entities.insert(key.into(), val); }
    pub fn emit_event(&mut self, ev: serde_json::Value) { self.events.push(ev); }
    pub fn log(&mut self, msg: impl Into<String>) { self.log.push(format!("[tick {}] {}", self.tick.0, msg.into())); }
    pub fn advance(&mut self) -> u64 { self.tick.advance() }
    pub fn entity_count(&self) -> usize { self.entities.len() }
    pub fn event_count(&self) -> usize { self.events.len() }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn ticks()    { let mut w = SimWorld::new(); w.advance(); w.advance(); assert_eq!(w.tick.0, 2); }
    #[test] fn entities() { let mut w = SimWorld::new(); w.set_entity("T-1", serde_json::json!({})); assert_eq!(w.entity_count(), 1); }
}
