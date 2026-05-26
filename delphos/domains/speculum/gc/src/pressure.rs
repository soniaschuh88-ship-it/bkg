use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
pub enum GcPressure { None, Low, Medium, High, Critical }
impl GcPressure {
    pub fn from_event_count(n: u64) -> Self {
        if n < 10_000 { Self::None } else if n < 100_000 { Self::Low }
        else if n < 500_000 { Self::Medium } else if n < 2_000_000 { Self::High } else { Self::Critical }
    }
    pub fn should_compact(self) -> bool { matches!(self, Self::Medium | Self::High | Self::Critical) }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn levels() { assert_eq!(GcPressure::from_event_count(100), GcPressure::None); assert!(GcPressure::Critical.should_compact()); }
}
