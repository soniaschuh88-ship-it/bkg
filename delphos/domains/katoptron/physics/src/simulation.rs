use serde::{Deserialize,Serialize};
use crate::node::{NodeId,PhysicsNode};
use crate::forces::SpringForce;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize,Default)]
pub enum SimState{#[default]Idle,Running,Converged}

#[allow(clippy::derivable_impls)]
pub struct PhysicsSimulation{
    pub nodes:Vec<PhysicsNode>,
    pub edges:Vec<(NodeId,NodeId)>,
    pub state:SimState, pub tick:u64,
    pub damping:f64, pub dt:f64,
}
impl PhysicsSimulation{
    pub fn new()->Self{Self{nodes:vec![],edges:vec![],state:SimState::Idle,tick:0,damping:0.85,dt:0.016}}
    pub fn add_node(&mut self,n:PhysicsNode)->&mut Self{self.nodes.push(n);self}
    pub fn add_edge(&mut self,a:NodeId,b:NodeId)->&mut Self{self.edges.push((a,b));self}
    pub fn step(&mut self){
        self.tick+=1; self.state=SimState::Running;
        let spring=SpringForce{rest_length:100.0,stiffness:0.05};
        let edges=self.edges.clone();
        for(a_id,b_id) in &edges{
            let ai=self.nodes.iter().position(|n|&n.id==a_id);
            let bi=self.nodes.iter().position(|n|&n.id==b_id);
            if let(Some(ai),Some(bi))=(ai,bi){
                let b_pos=(self.nodes[bi].x,self.nodes[bi].y,self.nodes[bi].mass);
                let bnode=PhysicsNode::new("",b_pos.2,b_pos.0,b_pos.1);
                spring.apply(&mut self.nodes[ai],&bnode);
            }
        }
        let dt=self.dt; let damping=self.damping;
        for n in &mut self.nodes{n.integrate(dt,damping);}
        let max_v=self.nodes.iter().map(|n|(n.vx*n.vx+n.vy*n.vy).sqrt()).fold(0.0_f64,f64::max);
        if max_v<0.01{self.state=SimState::Converged;}
    }
    pub fn run(&mut self,max_ticks:u64){for _ in 0..max_ticks{if self.state==SimState::Converged{break;}self.step();}}
    pub fn positions(&self)->Vec<(String,f64,f64)>{self.nodes.iter().map(|n|(n.id.0.clone(),n.x,n.y)).collect()}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn converges(){
        let mut sim=PhysicsSimulation::new();
        sim.add_node(PhysicsNode::new("A",1.0,0.0,0.0).pinned());
        sim.add_node(PhysicsNode::new("B",1.0,200.0,0.0));
        sim.add_edge(NodeId::new("A"),NodeId::new("B"));
        sim.run(200);
        assert_eq!(sim.state,SimState::Converged);
    }
    #[test] fn positions_returned(){
        let mut sim=PhysicsSimulation::new();
        sim.add_node(PhysicsNode::new("X",1.0,50.0,50.0));
        assert_eq!(sim.positions().len(),1);
    }
}

impl Default for PhysicsSimulation{fn default()->Self{Self::new()}}
