use std::collections::HashMap;
use bkg_core::{BkgError,BkgResult};
use petgraph::graph::{DiGraph,NodeIndex};
use crate::node::{MemoryEdge,MemoryNode,MemoryState};
pub struct MemoryGraph{graph:DiGraph<MemoryNode,MemoryEdge>,index:HashMap<String,NodeIndex>}
impl MemoryGraph{
    pub fn new()->Self{Self{graph:DiGraph::new(),index:HashMap::new()}}
    pub fn add_node(&mut self,n:MemoryNode)->BkgResult<NodeIndex>{if self.index.contains_key(&n.id){return Err(BkgError::Internal(format!("node '{}' exists",n.id)));}let id=n.id.clone();let nx=self.graph.add_node(n);self.index.insert(id,nx);Ok(nx)}
    pub fn add_edge(&mut self,from:&str,to:&str,e:MemoryEdge)->BkgResult<()>{let f=*self.index.get(from).ok_or_else(||BkgError::MemoryNodeNotFound(from.into()))?;let t=*self.index.get(to).ok_or_else(||BkgError::MemoryNodeNotFound(to.into()))?;self.graph.add_edge(f,t,e);Ok(())}
    pub fn recall(&mut self,id:&str)->BkgResult<()>{let nx=*self.index.get(id).ok_or_else(||BkgError::MemoryNodeNotFound(id.into()))?;self.graph[nx].recall();Ok(())}
    pub fn crystallize(&mut self,id:&str)->BkgResult<()>{let nx=*self.index.get(id).ok_or_else(||BkgError::MemoryNodeNotFound(id.into()))?;self.graph[nx].crystallize();Ok(())}
    pub fn fossilize(&mut self,id:&str)->BkgResult<()>{let nx=*self.index.get(id).ok_or_else(||BkgError::MemoryNodeNotFound(id.into()))?;self.graph[nx].fossilize();Ok(())}
    pub fn decay_tick(&mut self){for nx in self.graph.node_indices().collect::<Vec<_>>(){self.graph[nx].apply_decay();}}
    pub fn decay(&mut self,ticks:u32){for _ in 0..ticks{self.decay_tick();}}
    pub fn compress_decayed(&mut self)->Vec<String>{let rm:Vec<_>=self.graph.node_indices().filter(|&nx|self.graph[nx].state==MemoryState::Decayed).map(|nx|(nx,self.graph[nx].id.clone())).collect();let ids:Vec<_>=rm.iter().map(|(_,id)|id.clone()).collect();for(nx,id)in&rm{self.graph.remove_node(*nx);self.index.remove(id);}ids}
    pub fn get(&self,id:&str)->BkgResult<&MemoryNode>{let nx=*self.index.get(id).ok_or_else(||BkgError::MemoryNodeNotFound(id.into()))?;Ok(&self.graph[nx])}
    pub fn top_k(&self,k:usize)->Vec<&MemoryNode>{let mut ns:Vec<_>=self.graph.node_indices().map(|nx|&self.graph[nx]).filter(|n|n.state!=MemoryState::Decayed).collect();ns.sort_by(|a,b|b.importance.partial_cmp(&a.importance).unwrap());ns.truncate(k);ns}
    pub fn all_nodes(&self)->Vec<&MemoryNode>{self.graph.node_indices().map(|nx|&self.graph[nx]).collect()}
    pub fn node_count(&self)->usize{self.graph.node_count()}
    pub fn edge_count(&self)->usize{self.graph.edge_count()}
}
impl Default for MemoryGraph{fn default()->Self{Self::new()}}
#[cfg(test)]mod tests{use super::*;
    fn n(id:&str)->MemoryNode{MemoryNode::new(id,serde_json::json!({}),0.5,2,0.1)}
    #[test]fn add_get(){let mut g=MemoryGraph::new();g.add_node(n("a")).unwrap();assert_eq!(g.get("a").unwrap().id,"a");}
    #[test]fn dup_fails(){let mut g=MemoryGraph::new();g.add_node(n("a")).unwrap();assert!(g.add_node(n("a")).is_err());}
    #[test]fn decay(){let mut g=MemoryGraph::new();g.add_node(n("a")).unwrap();let b=g.get("a").unwrap().importance;g.decay_tick();assert!(g.get("a").unwrap().importance<b);}
    #[test]fn top_k(){let mut g=MemoryGraph::new();g.add_node(MemoryNode::new("lo",serde_json::json!({}),0.1,1,0.0)).unwrap();g.add_node(MemoryNode::new("hi",serde_json::json!({}),0.9,5,0.0)).unwrap();assert_eq!(g.top_k(1)[0].id,"hi");}
    #[test]fn compress(){let mut g=MemoryGraph::new();g.add_node(MemoryNode::new("d",serde_json::json!({}),0.05,1,1.0)).unwrap();g.add_node(n("a")).unwrap();g.decay_tick();let rm=g.compress_decayed();assert!(rm.contains(&"d".to_string()));assert_eq!(g.node_count(),1);}
}
