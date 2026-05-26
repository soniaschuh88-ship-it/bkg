use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
pub enum TaskStatus { #[default] Planning, Todo, InProgress, InReview, Done, Archived }
impl TaskStatus {
    pub fn is_terminal(self) -> bool { matches!(self, Self::Done | Self::Archived) }
    pub fn as_str(self) -> &'static str { match self { Self::Planning=>"planning", Self::Todo=>"todo", Self::InProgress=>"in-progress", Self::InReview=>"in-review", Self::Done=>"done", Self::Archived=>"archived" } }
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!((self, next),
            (Self::Planning, Self::Todo) | (Self::Planning, Self::InProgress) |
            (Self::Todo, Self::InProgress) | (Self::InProgress, Self::InReview) |
            (Self::InReview, Self::InProgress) | (Self::InReview, Self::Done) |
            (Self::Done, Self::Archived) | (_, Self::Archived)
        )
    }
}
impl std::fmt::Display for TaskStatus { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_str()) } }
#[derive(Debug, thiserror::Error)]
pub enum TaskTransitionError {
    #[error("invalid transition: {from} → {to}")]
    Invalid { from: TaskStatus, to: TaskStatus },
    #[error("task is terminal: {0}")]
    Terminal(TaskStatus),
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn valid_transitions() { assert!(TaskStatus::Planning.can_transition_to(TaskStatus::Todo)); assert!(TaskStatus::InProgress.can_transition_to(TaskStatus::InReview)); }
    #[test] fn terminal() { assert!(TaskStatus::Done.is_terminal()); assert!(!TaskStatus::InProgress.is_terminal()); }
    #[test] fn display() { assert_eq!(TaskStatus::InProgress.to_string(), "in-progress"); }
}