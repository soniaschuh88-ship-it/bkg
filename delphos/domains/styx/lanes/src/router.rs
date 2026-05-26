use std::collections::{HashMap, VecDeque};
use crate::{backpressure::BackpressureController,lane::{Lane,LaneClass},packet::{BusPacket,PacketStatus},qos::QosPolicy};
use bkg_core::{BkgError,BkgResult};
#[derive(Debug,Default)]
pub struct LaneRouter{queues:HashMap<String,VecDeque<BusPacket>>,bp:BackpressureController}
impl LaneRouter{
    pub fn new()->Self{Self::default()}
    pub fn send(&mut self,mut p:BusPacket)->BkgResult<()>{
        let l=Lane::new(&p.source_realm,&p.target_realm,p.lane_class);
        let q=QosPolicy::for_class(l.class);
        if q.would_drop(self.bp.depth(&l)){p.status=PacketStatus::Dropped;return Ok(());}
        if q.would_block(self.bp.depth(&l)){return Err(BkgError::Internal(format!("lane {} full",l.key())));}
        self.bp.record_enqueue(&l);
        self.queues.entry(l.key()).or_default().push_back(p);
        Ok(())
    }
    pub fn recv(&mut self,target:&str)->Option<BusPacket>{
        for class in [LaneClass::Critical,LaneClass::High,LaneClass::Normal,LaneClass::Background]{
            let suffix=format!("→{target}:{class}");
            for key in self.queues.keys().cloned().collect::<Vec<_>>(){
                if key.ends_with(&suffix){
                    if let Some(q)=self.queues.get_mut(&key){
                        if let Some(mut p)=q.pop_front(){
                            let l=Lane::new(&p.source_realm,&p.target_realm,p.lane_class);
                            self.bp.record_dequeue(&l);
                            p.mark_delivered();
                            return Some(p);
                        }
                    }
                }
            }
        }
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::BusPacket;
    #[test] fn send_recv(){let mut r=LaneRouter::new();r.send(BusPacket::new("a","b",LaneClass::Normal,1,1,"e",serde_json::json!({}))).unwrap();let p=r.recv("b").unwrap();assert_eq!(p.status,PacketStatus::Delivered);}
    #[test] fn priority(){let mut r=LaneRouter::new();r.send(BusPacket::new("a","b",LaneClass::Normal,1,1,"n",serde_json::json!({}))).unwrap();r.send(BusPacket::new("a","b",LaneClass::Critical,2,1,"c",serde_json::json!({}))).unwrap();assert_eq!(r.recv("b").unwrap().lane_class,LaneClass::Critical);}
    #[test] fn empty(){let mut r=LaneRouter::new();assert!(r.recv("x").is_none());}
}
