use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct NodeId(pub String);
impl NodeId{pub fn new(s:impl Into<String>)->Self{Self(s.into())}}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PhysicsNode{
    pub id:NodeId, pub mass:f64,
    pub x:f64, pub y:f64, pub vx:f64, pub vy:f64,
    pub pinned:bool,
}
impl PhysicsNode{
    pub fn new(id:impl Into<String>,mass:f64,x:f64,y:f64)->Self{Self{id:NodeId::new(id),mass,x,y,vx:0.0,vy:0.0,pinned:false}}
    pub fn pinned(mut self)->Self{self.pinned=true;self}
    pub fn apply_force(&mut self,fx:f64,fy:f64){if!self.pinned{self.vx+=fx/self.mass;self.vy+=fy/self.mass;}}
    pub fn integrate(&mut self,dt:f64,damping:f64){if self.pinned{return;}self.x+=self.vx*dt;self.y+=self.vy*dt;self.vx*=damping;self.vy*=damping;}
}
