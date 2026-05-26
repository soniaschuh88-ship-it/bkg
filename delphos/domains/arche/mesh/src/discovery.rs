use std::collections::BTreeMap;
use crate::node::{MeshNode, MeshNodeId, NodeStatus};

#[derive(Debug,Default)]
pub struct NodeRegistry{nodes:BTreeMap<String,MeshNode>}
impl NodeRegistry{
    pub fn new()->Self{Self::default()}
    pub fn register(&mut self,n:MeshNode){self.nodes.insert(n.id.0.clone(),n);}
    pub fn get(&self,id:&MeshNodeId)->Option<&MeshNode>{self.nodes.get(&id.0)}
    pub fn get_mut(&mut self,id:&MeshNodeId)->Option<&mut MeshNode>{self.nodes.get_mut(&id.0)}
    pub fn online_nodes(&self)->Vec<&MeshNode>{self.nodes.values().filter(|n|n.status.is_healthy()).collect()}
    pub fn all_nodes(&self)->Vec<&MeshNode>{self.nodes.values().collect()}
    pub fn count(&self)->usize{self.nodes.len()}
    pub fn remove(&mut self,id:&MeshNodeId)->Option<MeshNode>{self.nodes.remove(&id.0)}
    pub fn mark_stale(&mut self,timeout_secs:i64)->usize{
        let mut n=0;
        for node in self.nodes.values_mut(){
            if node.status==NodeStatus::Online && node.seconds_since_seen()>timeout_secs{
                node.mark_offline(); n+=1;
            }
        }
        n
    }
}

#[cfg(test)]
mod tests{use super::*;
    #[test] fn register_and_find(){
        let mut r=NodeRegistry::new();
        let mut n=MeshNode::new("n0","127.0.0.1:9000"); n.mark_online();
        let id=n.id.clone(); r.register(n);
        assert!(r.get(&id).is_some());
        assert_eq!(r.online_nodes().len(),1);
    }
    #[test] fn remove(){
        let mut r=NodeRegistry::new();
        let n=MeshNode::new("n","a"); let id=n.id.clone(); r.register(n);
        assert!(r.remove(&id).is_some());
        assert_eq!(r.count(),0);
    }
}
