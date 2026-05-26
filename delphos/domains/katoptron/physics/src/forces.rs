use crate::node::PhysicsNode;
pub struct GravityForce{pub g:f64}
impl GravityForce{
    pub fn apply(&self,a:&mut PhysicsNode,b:&PhysicsNode){
        let dx=b.x-a.x; let dy=b.y-a.y;
        let dist=(dx*dx+dy*dy).sqrt().max(1.0);
        let f=self.g*a.mass*b.mass/(dist*dist);
        let fx=f*dx/dist; let fy=f*dy/dist;
        a.apply_force(fx,fy);
    }
}
pub struct SpringForce{pub rest_length:f64,pub stiffness:f64}
impl SpringForce{
    pub fn apply(&self,a:&mut PhysicsNode,b:&PhysicsNode){
        let dx=b.x-a.x; let dy=b.y-a.y;
        let dist=(dx*dx+dy*dy).sqrt().max(0.001);
        let stretch=dist-self.rest_length;
        let f=self.stiffness*stretch;
        let fx=f*dx/dist; let fy=f*dy/dist;
        a.apply_force(fx,fy);
    }
}
