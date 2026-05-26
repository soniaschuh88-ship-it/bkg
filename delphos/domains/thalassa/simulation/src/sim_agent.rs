use crate::world::SimWorld;
pub struct SimAgent { pub name: String, pub script: Vec<String> }
impl SimAgent {
    pub fn new(name: impl Into<String>) -> Self { Self { name: name.into(), script: vec![] } }
    pub fn with_action(mut self, a: impl Into<String>) -> Self { self.script.push(a.into()); self }
    pub fn run(&self, world: &mut SimWorld) {
        for a in &self.script { world.log(format!("{}: {}", self.name, a)); world.advance(); }
    }
}
