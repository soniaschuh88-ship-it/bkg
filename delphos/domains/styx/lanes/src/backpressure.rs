use std::collections::HashMap;
use crate::lane::Lane;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum BackpressureSignal{Ok,Slow,Backoff,Drop}
#[derive(Debug,Default)]
pub struct BackpressureController{depths:HashMap<String,usize>}
impl BackpressureController{
    pub fn new()->Self{Self::default()}
    pub fn record_enqueue(&mut self,l:&Lane){*self.depths.entry(l.key()).or_insert(0)+=1;}
    pub fn record_dequeue(&mut self,l:&Lane){let e=self.depths.entry(l.key()).or_insert(0);if*e>0{*e-=1;}}
    pub fn signal(&self,l:&Lane)->BackpressureSignal{
        let d=self.depths.get(&l.key()).copied().unwrap_or(0);
        let cap=l.class.capacity();
        if d==0{return BackpressureSignal::Ok;}
        let r=d as f64/cap as f64;
        if r<0.5{BackpressureSignal::Ok}else if r<0.75{BackpressureSignal::Slow}else if r<1.0{BackpressureSignal::Backoff}else{BackpressureSignal::Drop}
    }
    pub fn depth(&self,l:&Lane)->usize{self.depths.get(&l.key()).copied().unwrap_or(0)}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ok_empty(){let mut b=BackpressureController::new();let l=Lane::new("a","b",LaneClass::Normal);assert_eq!(b.signal(&l),BackpressureSignal::Ok);}
    #[test] fn drop_at_cap(){let mut b=BackpressureController::new();let l=Lane::new("a","b",LaneClass::Normal);for _ in 0..=l.class.capacity(){b.record_enqueue(&l);}assert_eq!(b.signal(&l),BackpressureSignal::Drop);}
    #[test] fn dequeue(){let mut b=BackpressureController::new();let l=Lane::new("a","b",LaneClass::High);b.record_enqueue(&l);b.record_enqueue(&l);b.record_dequeue(&l);assert_eq!(b.depth(&l),1);}
}
