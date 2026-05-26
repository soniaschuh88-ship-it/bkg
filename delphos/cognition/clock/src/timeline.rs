use serde::{Deserialize, Serialize};
use crate::tick::SequencedInstant;

/// Ordered sequence of ticks for a realm.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Timeline { ticks: Vec<SequencedInstant> }
impl Timeline {
    pub fn new() -> Self { Self::default() }
    pub fn push(&mut self, t: SequencedInstant) { self.ticks.push(t); }
    pub fn len(&self) -> usize { self.ticks.len() }
    pub fn is_empty(&self) -> bool { self.ticks.is_empty() }
    pub fn latest(&self) -> Option<&SequencedInstant> { self.ticks.last() }
    pub fn from_offset(&self, lamport: u64) -> impl Iterator<Item=&SequencedInstant> {
        self.ticks.iter().filter(move |t| t.lamport >= lamport)
    }
}
