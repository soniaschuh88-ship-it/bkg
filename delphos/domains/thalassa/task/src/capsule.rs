//! Task Capsule — filesystem layout for isolated task execution.
//! Layout: .bkg/tasks/{id}/ with ledger/, diffs/, memory/, snapshots/, prompt.md, logs/
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::task::TaskId;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCapsule { pub task_id: TaskId, pub root: PathBuf }
impl TaskCapsule {
    pub fn new(task_id: TaskId, base_dir: impl AsRef<Path>) -> Self {
        let root = base_dir.as_ref().join(format!("{}", task_id));
        Self { task_id, root }
    }
    pub fn create_dirs(&self) -> std::io::Result<()> {
        for dir in &["ledger","diffs","memory","snapshots","logs"] {
            std::fs::create_dir_all(self.root.join(dir))?;
        }
        Ok(())
    }
    pub fn prompt_md_path(&self) -> PathBuf { self.root.join("prompt.md") }
    pub fn ledger_path(&self) -> PathBuf { self.root.join("ledger") }
    pub fn exists(&self) -> bool { self.root.exists() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn paths() { let c=TaskCapsule::new(TaskId("T-1".into()),"/tmp"); assert!(c.prompt_md_path().to_string_lossy().contains("prompt.md")); assert!(c.ledger_path().to_string_lossy().contains("ledger")); }
}