//! bkg-chat — chat rooms, mailbox, SSE streaming.
//! Single source of truth for all messaging in DELPHOS.
pub mod mailbox; pub mod message; pub mod room;
pub use message::{ChatMessage, MessageId};
pub use room::{ChatRoom, RoomId};
pub use mailbox::Mailbox;
