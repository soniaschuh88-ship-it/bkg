use std::collections::HashMap;
use crate::project::{Project, ProjectId};
#[derive(Debug, Default)]
pub struct ProjectRegistry { projects: HashMap<ProjectId, Project>, default_id: Option<ProjectId> }
impl ProjectRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, p: Project) -> &ProjectId { let id = p.id.clone(); self.projects.insert(id.clone(), p); let id_ref = self.projects.keys().find(|k| **k == id).unwrap(); id_ref }
    pub fn get(&self, id: &ProjectId) -> Option<&Project> { self.projects.get(id) }
    pub fn set_default(&mut self, id: ProjectId) { self.default_id = Some(id); }
    pub fn default_project(&self) -> Option<&Project> { self.default_id.as_ref().and_then(|id| self.projects.get(id)) }
    pub fn list(&self) -> Vec<&Project> { let mut v: Vec<_>=self.projects.values().collect(); v.sort_by(|a,b|a.name.cmp(&b.name)); v }
    pub fn count(&self) -> usize { self.projects.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    #[test] fn register_and_list() { let mut r=ProjectRegistry::new(); r.register(Project::new("A","/a")); r.register(Project::new("B","/b")); assert_eq!(r.count(),2); let list=r.list(); assert_eq!(list[0].name,"A"); }
    #[test] fn default_project() { let mut r=ProjectRegistry::new(); let p=Project::new("X","/x"); let id=p.id.clone(); r.register(p); r.set_default(id); assert!(r.default_project().is_some()); }
}