use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum MemoryState{Active,Crystallized,Fossilized,Decayed}
impl MemoryState{pub fn can_decay(self)->bool{matches!(self,MemoryState::Active)}pub fn is_mutable(self)->bool{!matches!(self,MemoryState::Fossilized)}}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MemoryNode{pub id:String,pub content:serde_json::Value,pub importance:f64,pub impact:f64,pub recurrence:u32,pub causal_depth:u32,pub decay_rate:f64,pub state:MemoryState,pub tags:Vec<String>,pub created_at:chrono::DateTime<chrono::Utc>,pub last_accessed:chrono::DateTime<chrono::Utc>}
impl MemoryNode{
    pub fn new(id:impl Into<String>,content:serde_json::Value,impact:f64,depth:u32,decay:f64)->Self{let imp=impact.clamp(0.0,1.0);Self{id:id.into(),content,importance:imp*depth as f64,impact:imp,recurrence:1,causal_depth:depth,decay_rate:decay.clamp(0.0,1.0),state:MemoryState::Active,tags:Vec::new(),created_at:chrono::Utc::now(),last_accessed:chrono::Utc::now()}}
    pub fn compute_importance(&mut self){self.importance=self.impact*self.recurrence as f64*self.causal_depth as f64;}
    pub fn recall(&mut self){if self.state.is_mutable(){self.recurrence=self.recurrence.saturating_add(1);self.last_accessed=chrono::Utc::now();self.compute_importance();}}
    pub fn apply_decay(&mut self){if!self.state.can_decay(){return;}self.impact=(self.impact-self.decay_rate).max(0.0);if self.impact<=f64::EPSILON{self.state=MemoryState::Decayed;self.importance=0.0;}else{self.compute_importance();}}
    pub fn crystallize(&mut self)->bool{if self.state==MemoryState::Active{self.state=MemoryState::Crystallized;true}else{false}}
    pub fn fossilize(&mut self)->bool{if!matches!(self.state,MemoryState::Fossilized|MemoryState::Decayed){self.state=MemoryState::Fossilized;true}else{false}}
    pub fn with_tags(mut self,tags:impl IntoIterator<Item=impl Into<String>>)->Self{self.tags=tags.into_iter().map(Into::into).collect();self}
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MemoryEdge{pub relation:String,pub weight:f64}
impl MemoryEdge{pub fn new(r:impl Into<String>,w:f64)->Self{Self{relation:r.into(),weight:w.clamp(0.0,1.0)}}}
#[cfg(test)]mod tests{use super::*;
    #[test]fn importance(){let mut n=MemoryNode::new("n",serde_json::json!({}),0.5,4,0.1);assert!((n.importance-2.0).abs()<1e-9);n.recall();assert!((n.importance-4.0).abs()<1e-9);}
    #[test]fn decay(){let mut n=MemoryNode::new("n",serde_json::json!({}),1.0,1,0.1);let b=n.importance;n.apply_decay();assert!(n.importance<b);}
    #[test]fn crystal_no_decay(){let mut n=MemoryNode::new("n",serde_json::json!({}),1.0,1,0.5);n.crystallize();let b=n.importance;n.apply_decay();assert!((n.importance-b).abs()<1e-9);}
    #[test]fn full_decay(){let mut n=MemoryNode::new("n",serde_json::json!({}),0.1,1,1.0);n.apply_decay();assert_eq!(n.state,MemoryState::Decayed);}
}
