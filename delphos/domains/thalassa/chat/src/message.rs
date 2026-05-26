use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);
impl MessageId { pub fn new() -> Self { Self(uuid::Uuid::new_v4().to_string()) } }
impl Default for MessageId { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: MessageId, pub room_id: Option<String>, pub sender_id: String, pub sender_name: String,
    pub content: String, pub attachments: Vec<String>, pub mentions: Vec<String>,
    pub sent_at: DateTime<Utc>, pub edited: bool,
}
impl ChatMessage {
    pub fn new(sender_id: impl Into<String>, sender_name: impl Into<String>, content: impl Into<String>) -> Self {
        Self { id: MessageId::new(), room_id: None, sender_id: sender_id.into(), sender_name: sender_name.into(), content: content.into(), attachments: vec![], mentions: vec![], sent_at: Utc::now(), edited: false }
    }
    pub fn in_room(mut self, room_id: impl Into<String>) -> Self { self.room_id = Some(room_id.into()); self }
    pub fn with_mention(mut self, user: impl Into<String>) -> Self { self.mentions.push(user.into()); self }
    pub fn is_mention(&self, user_id: &str) -> bool { self.mentions.iter().any(|m| m == user_id) }
}
