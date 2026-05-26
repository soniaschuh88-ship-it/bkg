use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
pub enum AccessPolicy { #[default] Auto, Prompt, Deny }
impl AccessPolicy {
    pub fn as_str(self) -> &'static str { match self { Self::Auto=>"auto", Self::Prompt=>"prompt", Self::Deny=>"deny" } }
    pub fn would_auto_allow(self) -> bool { matches!(self, Self::Auto) }
}
