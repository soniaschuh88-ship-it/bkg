use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum GraphChange { NodeAdded(String), NodeRemoved(String), EdgeAdded(String,String), EdgeRemoved(String,String) }
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct GraphDiff { pub changes: Vec<GraphChange> }
impl GraphDiff { pub fn new() -> Self { Self::default() } pub fn push(&mut self, c: GraphChange) { self.changes.push(c); } pub fn is_empty(&self) -> bool { self.changes.is_empty() } }
