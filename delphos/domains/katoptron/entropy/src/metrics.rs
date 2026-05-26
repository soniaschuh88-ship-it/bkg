use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MetricSnapshot{
    pub entropy:f64, pub pressure:f64, pub heat:f64, pub stability:f64,
    pub latency_p95_ms:f64, pub drift_rate:f64, pub timestamp:DateTime<Utc>,
}
impl MetricSnapshot{
    #[allow(clippy::too_many_arguments)]
    pub fn compute(total_nodes:usize,blocked:usize,active_agents:usize,max_agents:usize,error_rate:f64,latency_p95_ms:f64,hash_drift_count:u64,total_events:u64)->Self{
        let entropy=if total_nodes==0{0.0}else{(blocked as f64/total_nodes as f64).clamp(0.0,1.0)};
        let pressure=if max_agents==0{0.0}else{(active_agents as f64/max_agents as f64).clamp(0.0,1.0)};
        let heat=(error_rate*10.0).clamp(0.0,1.0);
        let stability=1.0-heat-entropy*0.5;
        let drift_rate=if total_events==0{0.0}else{hash_drift_count as f64/total_events as f64};
        Self{entropy,pressure,heat,stability:stability.clamp(0.0,1.0),latency_p95_ms,drift_rate,timestamp:Utc::now()}
    }
    pub fn health_label(&self)->&'static str{
        if self.stability>0.8{"healthy"}else if self.stability>0.5{"degraded"}else{"critical"}
    }
}
#[derive(Debug,Default)]
pub struct SystemMetrics{snapshots:Vec<MetricSnapshot>}
impl SystemMetrics{
    pub fn new()->Self{Self::default()}
    pub fn record(&mut self,s:MetricSnapshot){self.snapshots.push(s);}
    pub fn latest(&self)->Option<&MetricSnapshot>{self.snapshots.last()}
    pub fn avg_entropy(&self)->f64{if self.snapshots.is_empty(){0.0}else{self.snapshots.iter().map(|s|s.entropy).sum::<f64>()/self.snapshots.len() as f64}}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn healthy(){let s=MetricSnapshot::compute(10,0,2,8,0.0,50.0,0,1000);assert_eq!(s.health_label(),"healthy");}
    #[test] fn critical(){let s=MetricSnapshot::compute(10,10,8,8,0.5,1000.0,100,200);assert!(s.stability<0.5);}
    #[test] fn avg(){let mut m=SystemMetrics::new();m.record(MetricSnapshot::compute(10,2,2,8,0.0,50.0,0,100));m.record(MetricSnapshot::compute(10,4,4,8,0.0,50.0,0,100));assert!(m.avg_entropy()>0.0);}
}
