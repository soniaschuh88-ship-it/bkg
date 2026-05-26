use std::collections::BTreeMap;
#[derive(Debug,Clone)]
pub struct GeometryNode{pub id:String,pub x:f64,pub y:f64,pub w:f64,pub h:f64}
#[derive(Debug,Clone,Default)]
pub struct GeometryGraph{pub nodes:BTreeMap<String,GeometryNode>}
impl GeometryGraph{
    pub fn new()->Self{Self::default()}
    pub fn place(&mut self,id:impl Into<String>,x:f64,y:f64,w:f64,h:f64){let id=id.into();self.nodes.insert(id.clone(),GeometryNode{id,x,y,w,h});}
    pub fn get(&self,id:&str)->Option<&GeometryNode>{self.nodes.get(id)}
}
