use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::message::ChatMessage;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub String);
impl RoomId { pub fn new() -> Self { Self(format!("ROOM-{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase())) } }
impl Default for RoomId { fn default() -> Self { Self::new() } }
impl std::fmt::Display for RoomId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }

/// A chat room with members and message history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: RoomId, pub name: String, pub members: Vec<String>,
    pub direct_responders: Vec<String>, pub ambient_responders: Vec<String>,
    pub messages: Vec<ChatMessage>, pub created_at: DateTime<Utc>,
}
impl ChatRoom {
    pub fn new(name: impl Into<String>) -> Self { Self { id: RoomId::new(), name: name.into(), members: vec![], direct_responders: vec![], ambient_responders: vec![], messages: vec![], created_at: Utc::now() } }
    pub fn add_member(&mut self, id: impl Into<String>) { self.members.push(id.into()); }
    pub fn post(&mut self, mut msg: ChatMessage) -> &ChatMessage {
        msg.room_id = Some(self.id.0.clone());
        self.messages.push(msg);
        self.messages.last().unwrap()
    }
    pub fn messages_for(&self, member_id: &str) -> Vec<&ChatMessage> { self.messages.iter().filter(|m| m.sender_id==member_id || m.mentions.is_empty() || m.is_mention(member_id)).collect() }
    pub fn member_count(&self) -> usize { self.members.len() }
    pub fn message_count(&self) -> usize { self.messages.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ChatMessage;
    #[test] fn post_message() { let mut r=ChatRoom::new("general"); r.add_member("alice"); r.add_member("bob"); r.post(ChatMessage::new("alice","Alice","hello everyone!")); assert_eq!(r.message_count(),1); }
    #[test] fn mentions() { let mut r=ChatRoom::new("team"); r.post(ChatMessage::new("alice","Alice","hey").with_mention("bob")); let msgs=r.messages_for("bob"); assert_eq!(msgs.len(),1); }
}
