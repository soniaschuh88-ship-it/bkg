use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
pub enum SchedulerPriority{Background=0,Low=1,Normal=2,High=3,Urgent=4}
impl SchedulerPriority{pub fn as_str(self)->&'static str{match self{Self::Background=>"background",Self::Low=>"low",Self::Normal=>"normal",Self::High=>"high",Self::Urgent=>"urgent"}}}
#[allow(clippy::derivable_impls)]
impl Default for SchedulerPriority{fn default()->Self{Self::Normal}}
