use std::collections::HashMap;
use bkg_core::BkgResult;
use tokio::sync::broadcast;
pub struct EventBus{channels:HashMap<String,broadcast::Sender<serde_json::Value>>,cap:usize}
impl EventBus{
    pub fn new()->Self{Self{channels:HashMap::new(),cap:256}}
    pub fn publish(&mut self,topic:&str,payload:serde_json::Value)->BkgResult<usize>{let s=self.channels.entry(topic.to_string()).or_insert_with(||broadcast::channel(self.cap).0);Ok(s.send(payload).unwrap_or(0))}
    pub fn subscribe(&mut self,topic:&str)->broadcast::Receiver<serde_json::Value>{self.channels.entry(topic.to_string()).or_insert_with(||broadcast::channel(self.cap).0).subscribe()}
}
impl Default for EventBus{fn default()->Self{Self::new()}}
#[cfg(test)]mod tests{use super::*;
    #[tokio::test]async fn pub_sub(){let mut b=EventBus::new();let mut rx=b.subscribe("t");b.publish("t",serde_json::json!({"x":1})).unwrap();assert_eq!(rx.recv().await.unwrap()["x"],1);}
}
