use std::time::Duration;
use crate::lane::LaneClass;
#[derive(Debug,Clone)]
pub struct QosPolicy{pub class:LaneClass,pub max_queue_depth:usize,pub target_latency:Duration,pub drop_on_overflow:bool}
impl QosPolicy{
    pub fn for_class(c:LaneClass)->Self{Self{class:c,max_queue_depth:c.capacity(),target_latency:Duration::from_millis(c.latency_target_ms()),drop_on_overflow:matches!(c,LaneClass::Background)}}
    pub fn would_drop(&self,depth:usize)->bool{self.drop_on_overflow&&depth>=self.max_queue_depth}
    pub fn would_block(&self,depth:usize)->bool{!self.drop_on_overflow&&depth>=self.max_queue_depth}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn critical_no_drop(){assert!(!QosPolicy::for_class(LaneClass::Critical).drop_on_overflow);}
    #[test] fn bg_drops(){assert!(QosPolicy::for_class(LaneClass::Background).would_drop(512));}
}
