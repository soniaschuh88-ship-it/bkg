use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A live subscription to projection updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionSubscription {
    pub subscriber_id: String,
    pub realm_id: String,
    pub projection_ids: Vec<String>,
    pub from_version: u64,
    pub subscribed_at: DateTime<Utc>,
    pub active: bool,
}

impl ProjectionSubscription {
    pub fn new(subscriber_id: impl Into<String>, realm_id: impl Into<String>, projection_ids: Vec<String>) -> Self {
        Self { subscriber_id: subscriber_id.into(), realm_id: realm_id.into(), projection_ids, from_version: 0, subscribed_at: Utc::now(), active: true }
    }
    pub fn cancel(&mut self) { self.active = false; }
}

/// Manages active projection subscriptions.
#[derive(Debug, Default)]
pub struct ProjectionSubscriber {
    subscriptions: Vec<ProjectionSubscription>,
}

impl ProjectionSubscriber {
    pub fn new() -> Self { Self::default() }
    pub fn subscribe(&mut self, sub: ProjectionSubscription) { self.subscriptions.push(sub); }
    pub fn active_for_realm(&self, realm_id: &str) -> Vec<&ProjectionSubscription> {
        self.subscriptions.iter().filter(|s| s.active && s.realm_id == realm_id).collect()
    }
    pub fn count(&self) -> usize { self.subscriptions.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn subscribe_and_list() {
        let mut ps = ProjectionSubscriber::new();
        ps.subscribe(ProjectionSubscription::new("ui-1","telum",vec!["kanban".into()]));
        assert_eq!(ps.active_for_realm("telum").len(), 1);
        assert_eq!(ps.active_for_realm("styx").len(), 0);
    }
    #[test] fn cancel() {
        let mut ps = ProjectionSubscriber::new();
        let mut sub = ProjectionSubscription::new("ui-2","telum",vec![]);
        sub.cancel();
        ps.subscribe(sub);
        assert_eq!(ps.active_for_realm("telum").len(), 0);
    }
}
