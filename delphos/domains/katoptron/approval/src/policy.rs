use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
pub enum ActionPolicy { Allow, #[default] RequireApproval, Block }
impl ActionPolicy { pub fn as_str(self)->&'static str { match self{Self::Allow=>"allow",Self::RequireApproval=>"require-approval",Self::Block=>"block"} } }
