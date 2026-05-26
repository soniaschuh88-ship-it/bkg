use std::collections::{BTreeMap, VecDeque};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailItem { pub id: String, pub from: String, pub subject: String, pub body: serde_json::Value, pub received_at: DateTime<Utc>, pub read: bool }
impl MailItem {
    pub fn new(from: impl Into<String>, subject: impl Into<String>, body: serde_json::Value) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), from: from.into(), subject: subject.into(), body, received_at: Utc::now(), read: false }
    }
}

#[derive(Debug, Default)]
pub struct Mailbox { inboxes: BTreeMap<String, VecDeque<MailItem>> }
impl Mailbox {
    pub fn new() -> Self { Self::default() }
    pub fn deliver(&mut self, to: impl Into<String>, item: MailItem) { self.inboxes.entry(to.into()).or_default().push_back(item); }
    pub fn receive(&mut self, user_id: &str) -> Option<MailItem> {
        let inbox = self.inboxes.get_mut(user_id)?;
        let mut item = inbox.pop_front()?;
        item.read = true; Some(item)
    }
    pub fn unread_count(&self, user_id: &str) -> usize { self.inboxes.get(user_id).map(|q| q.iter().filter(|m| !m.read).count()).unwrap_or(0) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn deliver_receive() { let mut mb=Mailbox::new(); mb.deliver("alice",MailItem::new("system","New task","".into())); assert_eq!(mb.unread_count("alice"),1); let m=mb.receive("alice").unwrap(); assert!(m.read); assert_eq!(mb.unread_count("alice"),0); }
}
