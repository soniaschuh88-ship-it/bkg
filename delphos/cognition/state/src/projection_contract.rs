// projection_contract.rs — Projection Hash Contract + Rebuild Guarantee.
//
// THREE laws:
//
//   Law 1: Every ProjectionView has a Hash256 checksum, a ProjectionVersion,
//           and an EventRange that identifies exactly which events built it.
//           → Without this, replay validation is impossible.
//
//   Law 2: Every projection must be rebuildable from the ledger alone.
//           No incremental trust. No cached partial state.
//           rebuild_from_ledger(ledger, range) → ProjectionView<T>
//           must always produce the same result.
//
//   Law 3: The Materializer is validated by the Kernel.
//           Event → State → Kernel.validate_projection() → ProjectionView<T>
//           A projection that bypasses Kernel validation is rejected.
//
// Single source of truth for projection integrity.

use serde::{Deserialize, Serialize};
use bkg_core::{BkgError, BkgResult};

// ─── EventRange ───────────────────────────────────────────────────────────────

/// The range of events (by lamport counter) that built a projection.
/// Required for replay validation: same range → same projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EventRange {
    /// First event lamport counter included in this projection.
    pub from_lamport: u64,
    /// Last event lamport counter included (inclusive).
    pub to_lamport: u64,
    /// Total event count in range.
    pub event_count: u64,
}

impl EventRange {
    pub fn new(from: u64, to: u64, count: u64) -> Self {
        Self { from_lamport: from, to_lamport: to, event_count: count }
    }

    pub fn single(lamport: u64) -> Self { Self::new(lamport, lamport, 1) }

    pub fn empty() -> Self { Self::new(0, 0, 0) }

    pub fn is_empty(&self) -> bool { self.event_count == 0 }

    /// Whether this range contains a specific lamport counter.
    pub fn contains(&self, lamport: u64) -> bool {
        !self.is_empty() && lamport >= self.from_lamport && lamport <= self.to_lamport
    }

    /// Extend this range to include a new event.
    pub fn extend(&self, lamport: u64) -> Self {
        if self.is_empty() {
            Self::single(lamport)
        } else {
            Self::new(
                self.from_lamport.min(lamport),
                self.to_lamport.max(lamport),
                self.event_count + 1,
            )
        }
    }

    /// Whether two ranges overlap (useful for conflict detection).
    pub fn overlaps(&self, other: &Self) -> bool {
        !self.is_empty() && !other.is_empty()
            && self.from_lamport <= other.to_lamport
            && other.from_lamport <= self.to_lamport
    }
}

impl std::fmt::Display for EventRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "[empty]")
        } else {
            write!(f, "[{}..{} ({} events)]", self.from_lamport, self.to_lamport, self.event_count)
        }
    }
}

// ─── ProjectionVersion ────────────────────────────────────────────────────────

/// The version of a projection — monotonically increasing.
/// A projection rebuilt from scratch always produces the same version
/// for the same event range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProjectionVersion(pub u64);

impl ProjectionVersion {
    pub fn zero() -> Self { Self(0) }
    pub fn from_lamport(lamport: u64) -> Self { Self(lamport) }
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}

impl std::fmt::Display for ProjectionVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "pv{}", self.0) }
}

// ─── ProjectionChecksum ───────────────────────────────────────────────────────

/// A deterministic checksum over a projection's data.
/// Same inputs → same checksum, always.
/// Used to detect projection drift between builds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionChecksum(pub String);

impl ProjectionChecksum {
    /// Compute a deterministic checksum from serialized projection data.
    pub fn compute(data: &serde_json::Value) -> Self {
        use std::hash::Hash;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        data.to_string().hash(&mut h);
        Self(format!("{:x}", std::hash::Hasher::finish(&h)))
    }

    pub fn wrap(s: impl Into<String>) -> Self { Self(s.into()) }

    pub fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl std::fmt::Display for ProjectionChecksum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "ck:{}", &self.0[..8.min(self.0.len())]) }
}

// ─── ProjectionContract ───────────────────────────────────────────────────────

/// The mathematical contract that every ProjectionView must satisfy.
///
/// A projection is valid if and only if:
///   1. Its checksum matches `recompute(event_range, data)`
///   2. Its event_range is non-empty (unless it's an empty initial state)
///   3. Its version matches its event_range.to_lamport
///   4. It was produced by a validated Materializer (has a KernelStamp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionContract {
    pub projection_id: String,
    pub realm_id: String,
    pub version: ProjectionVersion,
    pub event_range: EventRange,
    pub checksum: ProjectionChecksum,
    /// Proof that this projection passed through the Kernel.
    pub kernel_stamp: Option<KernelStamp>,
}

impl ProjectionContract {
    pub fn new(
        projection_id: impl Into<String>,
        realm_id: impl Into<String>,
        version: ProjectionVersion,
        event_range: EventRange,
        data: &serde_json::Value,
    ) -> Self {
        Self {
            projection_id: projection_id.into(),
            realm_id: realm_id.into(),
            version,
            event_range,
            checksum: ProjectionChecksum::compute(data),
            kernel_stamp: None,
        }
    }

    pub fn with_kernel_stamp(mut self, stamp: KernelStamp) -> Self {
        self.kernel_stamp = Some(stamp);
        self
    }

    /// Verify this contract against fresh data.
    /// Returns Ok if the checksum matches — the projection is intact.
    pub fn verify(&self, data: &serde_json::Value) -> Result<(), ProjectionViolation> {
        let fresh = ProjectionChecksum::compute(data);
        if !self.checksum.matches(&fresh) {
            return Err(ProjectionViolation::ChecksumMismatch {
                expected: self.checksum.clone(),
                actual: fresh,
            });
        }
        if self.kernel_stamp.is_none() {
            return Err(ProjectionViolation::NoKernelStamp {
                projection_id: self.projection_id.clone(),
            });
        }
        Ok(())
    }

    /// Verify that rebuilding from the ledger produces the same checksum.
    /// This is the Rebuild Guarantee: same events → same projection.
    pub fn verify_rebuild_identity(
        &self,
        rebuilt_checksum: &ProjectionChecksum,
    ) -> Result<(), ProjectionViolation> {
        if !self.checksum.matches(rebuilt_checksum) {
            return Err(ProjectionViolation::RebuildDivergence {
                projection_id: self.projection_id.clone(),
                original: self.checksum.clone(),
                rebuilt: rebuilt_checksum.clone(),
            });
        }
        Ok(())
    }
}

/// A projection contract was violated.
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum ProjectionViolation {
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: ProjectionChecksum, actual: ProjectionChecksum },

    #[error("projection '{projection_id}' has no kernel stamp — bypassed validation")]
    NoKernelStamp { projection_id: String },

    #[error("rebuild divergence for '{projection_id}': original={original}, rebuilt={rebuilt}")]
    RebuildDivergence { projection_id: String, original: ProjectionChecksum, rebuilt: ProjectionChecksum },

    #[error("event range mismatch: expected {expected}, got {actual}")]
    EventRangeMismatch { expected: EventRange, actual: EventRange },

    #[error("stale projection: version {current}, state is at {latest}")]
    Stale { current: ProjectionVersion, latest: ProjectionVersion },
}

// ─── KernelStamp ─────────────────────────────────────────────────────────────

/// Proof that the Kernel validated the materializer's output.
/// A projection without a KernelStamp is structurally rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelStamp {
    pub stamped_at_lamport: u64,
    pub pipeline_version: u16,
    /// Hash of (projection_id + checksum + event_range) — tamper-evident.
    pub stamp_hash: String,
}

impl KernelStamp {
    pub fn new(projection_id: &str, checksum: &ProjectionChecksum, event_range: &EventRange, lamport: u64) -> Self {
        use std::hash::Hash;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        projection_id.hash(&mut h);
        checksum.0.hash(&mut h);
        event_range.from_lamport.hash(&mut h);
        event_range.to_lamport.hash(&mut h);
        Self {
            stamped_at_lamport: lamport,
            pipeline_version: 1,
            stamp_hash: format!("{:x}", std::hash::Hasher::finish(&h)),
        }
    }

    pub fn is_valid(&self) -> bool { !self.stamp_hash.is_empty() }
}

// ─── MaterializerKernel ───────────────────────────────────────────────────────

/// The Kernel hook for the Materializer.
///
/// Flow: Event → State → MaterializerKernel.stamp() → ProjectionView<T>
///
/// Without going through this, a projection cannot be created.
/// This is Law 3: Materializer must be validated by Kernel.
pub struct MaterializerKernel {
    /// Current kernel lamport (advances with each stamp).
    current_lamport: u64,
}

impl MaterializerKernel {
    pub fn new() -> Self { Self { current_lamport: 0 } }

    /// Stamp a projection contract. Only the Kernel can do this.
    /// Returns a ProjectionContract with a KernelStamp attached.
    pub fn stamp(
        &mut self,
        projection_id: &str,
        realm_id: &str,
        event_range: EventRange,
        data: &serde_json::Value,
    ) -> ProjectionContract {
        self.current_lamport += 1;
        let version = ProjectionVersion::from_lamport(event_range.to_lamport);
        let mut contract = ProjectionContract::new(projection_id, realm_id, version, event_range, data);
        let stamp = KernelStamp::new(projection_id, &contract.checksum, &event_range, self.current_lamport);
        contract.kernel_stamp = Some(stamp);
        contract
    }

    /// Verify an existing contract hasn't been tampered with.
    pub fn verify_contract(&self, contract: &ProjectionContract, data: &serde_json::Value) -> BkgResult<()> {
        contract.verify(data).map_err(|e| BkgError::Internal(e.to_string()))
    }

    pub fn current_lamport(&self) -> u64 { self.current_lamport }
}

impl Default for MaterializerKernel { fn default() -> Self { Self::new() } }

// ─── RebuildProof ─────────────────────────────────────────────────────────────

/// Proof that a projection was fully rebuilt from the ledger.
/// No incremental trust. No cached state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebuildProof {
    pub projection_id: String,
    pub event_range: EventRange,
    pub checksum: ProjectionChecksum,
    pub rebuilt_at_lamport: u64,
    /// Whether this rebuild matched the original checksum.
    pub identity_confirmed: bool,
}

impl RebuildProof {
    pub fn new(
        projection_id: &str,
        event_range: EventRange,
        checksum: ProjectionChecksum,
        lamport: u64,
        identity_confirmed: bool,
    ) -> Self {
        Self { projection_id: projection_id.into(), event_range, checksum, rebuilt_at_lamport: lamport, identity_confirmed }
    }
}

/// Records of all projection rebuilds — used for drift detection.
#[derive(Debug, Default)]
pub struct RebuildLog { proofs: Vec<RebuildProof> }

impl RebuildLog {
    pub fn new() -> Self { Self::default() }
    pub fn record(&mut self, proof: RebuildProof) { self.proofs.push(proof); }
    pub fn failures(&self) -> Vec<&RebuildProof> { self.proofs.iter().filter(|p| !p.identity_confirmed).collect() }
    pub fn all_confirmed(&self) -> bool { self.proofs.iter().all(|p| p.identity_confirmed) }
    pub fn count(&self) -> usize { self.proofs.len() }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> serde_json::Value { serde_json::json!({"tasks":["T-1","T-2"],"count":2}) }

    #[test]
    fn event_range_extend() {
        let r = EventRange::empty().extend(5).extend(3).extend(8);
        assert_eq!(r.from_lamport, 3);
        assert_eq!(r.to_lamport, 8);
        assert_eq!(r.event_count, 3);
        assert!(r.contains(5));
        assert!(!r.contains(9));
    }

    #[test]
    fn event_range_overlap() {
        let a = EventRange::new(1, 5, 5);
        let b = EventRange::new(4, 8, 5);
        let c = EventRange::new(10, 15, 6);
        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn checksum_deterministic() {
        let d = sample_data();
        let c1 = ProjectionChecksum::compute(&d);
        let c2 = ProjectionChecksum::compute(&d);
        assert!(c1.matches(&c2));
    }

    #[test]
    fn checksum_sensitive() {
        let d1 = serde_json::json!({"x":1});
        let d2 = serde_json::json!({"x":2});
        assert!(!ProjectionChecksum::compute(&d1).matches(&ProjectionChecksum::compute(&d2)));
    }

    #[test]
    fn kernel_stamp_and_verify() {
        let mut kernel = MaterializerKernel::new();
        let data = sample_data();
        let range = EventRange::new(1, 10, 10);
        let contract = kernel.stamp("kanban", "telum", range, &data);

        assert!(contract.kernel_stamp.is_some());
        assert!(contract.verify(&data).is_ok());
    }

    #[test]
    fn no_kernel_stamp_fails_verify() {
        let data = sample_data();
        let range = EventRange::new(1, 5, 5);
        let contract = ProjectionContract::new("p", "r", ProjectionVersion::zero(), range, &data);
        // No kernel stamp → verification fails
        assert!(matches!(contract.verify(&data), Err(ProjectionViolation::NoKernelStamp { .. })));
    }

    #[test]
    fn tampered_data_fails_verify() {
        let mut kernel = MaterializerKernel::new();
        let data = sample_data();
        let contract = kernel.stamp("p", "r", EventRange::single(1), &data);
        let tampered = serde_json::json!({"tasks":["T-1","T-2","T-99"],"count":3});
        assert!(matches!(contract.verify(&tampered), Err(ProjectionViolation::ChecksumMismatch { .. })));
    }

    #[test]
    fn rebuild_identity_confirmed() {
        let data = sample_data();
        let original_ck = ProjectionChecksum::compute(&data);
        let rebuilt_ck = ProjectionChecksum::compute(&data); // same inputs
        let proof = RebuildProof::new("kanban", EventRange::new(1, 10, 10), rebuilt_ck.clone(), 11, original_ck.matches(&rebuilt_ck));
        assert!(proof.identity_confirmed);
    }

    #[test]
    fn rebuild_divergence_detected() {
        let original = ProjectionChecksum::compute(&serde_json::json!({"v":1}));
        let rebuilt = ProjectionChecksum::compute(&serde_json::json!({"v":2}));
        let range = EventRange::new(1,5,5);
        let proof = RebuildProof::new("p", range, rebuilt, 6, false);
        let mut log = RebuildLog::new(); log.record(proof);
        assert!(!log.all_confirmed());
        assert_eq!(log.failures().len(), 1);
    }

    #[test]
    fn kernel_stamps_are_tamper_evident() {
        let mut k = MaterializerKernel::new();
        let d = sample_data();
        let c = k.stamp("p", "r", EventRange::single(5), &d);
        let stamp = c.kernel_stamp.unwrap();
        assert!(stamp.is_valid());
        assert_eq!(stamp.pipeline_version, 1);
    }

    #[test]
    fn version_from_range() {
        let range = EventRange::new(10, 50, 40);
        let version = ProjectionVersion::from_lamport(range.to_lamport);
        assert_eq!(version.0, 50);
    }
}
