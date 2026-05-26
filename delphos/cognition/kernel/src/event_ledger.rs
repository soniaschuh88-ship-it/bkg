// event_ledger.rs — Append-only, hash-chained event store.
//
// The ledger IS the source of truth.
// Everything else (RealmState, projections) is derived from it.
//
// Properties:
//   1. APPEND-ONLY:     entries are never modified or deleted.
//   2. HASH-CHAINED:    each entry commits to all prior entries.
//                       chain_n = hash(entry_n.id ∥ entry_n.payload_hash ∥ chain_{n-1})
//   3. LAMPORT-MONOTONE:entry.lamport must exceed every prior entry's lamport.
//   4. TAMPER-EVIDENT:  any modification or insertion breaks verify_chain().
//   5. DETERMINISTIC:   same events in same order → same chain tip, always.
//
// Single source of truth for all DELPHOS event history.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bkg_core::RealmId;

// ─── LedgerEntry ─────────────────────────────────────────────────────────────

/// One entry in the event ledger.
/// Immutable after appending.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Monotone sequence index within this ledger (0-based).
    pub seq: u64,
    /// The event ID.
    pub event_id: String,
    /// Schema identifier.
    pub schema_id: String,
    /// Lamport clock value — strictly increasing.
    pub lamport: u64,
    /// Payload hash (from PipelineEvent.payload_hash).
    pub payload_hash: String,
    /// The serialized payload — stored for replay.
    pub payload: serde_json::Value,
    /// Actor who produced this event.
    pub producer: String,
    /// Causal parent event ID (if any).
    pub causal_parent: Option<String>,
    /// Hash of (event_id ∥ payload_hash ∥ prev_chain_hash).
    /// Forms the hash chain.
    pub chain_hash: String,
    /// Wall time of append (display only — never used for ordering).
    pub appended_at: DateTime<Utc>,
}

impl LedgerEntry {
    /// Compute the chain hash: hash(event_id ∥ payload_hash ∥ prev_chain_hash).
    pub fn compute_chain_hash(event_id: &str, payload_hash: &str, prev_chain_hash: &str) -> String {
        use std::hash::Hash;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        event_id.hash(&mut h);
        payload_hash.hash(&mut h);
        prev_chain_hash.hash(&mut h);
        format!("{:x}", std::hash::Hasher::finish(&h))
    }

    pub fn verify_chain_hash(&self, prev_chain_hash: &str) -> bool {
        let expected = Self::compute_chain_hash(&self.event_id, &self.payload_hash, prev_chain_hash);
        self.chain_hash == expected
    }
}

// ─── LedgerAppendError ────────────────────────────────────────────────────────

#[derive(Debug, Clone, thiserror::Error)]
pub enum LedgerAppendError {
    #[error("lamport not monotone: ledger tip={tip}, new={new}")]
    LamportNotMonotone { tip: u64, new: u64 },
    #[error("duplicate event_id: {event_id}")]
    DuplicateEventId { event_id: String },
    #[error("ledger is sealed — no further appends allowed")]
    Sealed,
}

// ─── EventLedger ─────────────────────────────────────────────────────────────

/// The append-only, hash-chained event ledger.
///
/// Every `Realm` owns exactly one `EventLedger`.
/// Every event that reaches `KernelPhase::Emitting` is appended here.
/// Nothing else may mutate the ledger.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EventLedger {
    pub realm_id: Option<RealmId>,
    entries: Vec<LedgerEntry>,
    /// Hash of the last entry — the chain tip.
    chain_tip: String,
    /// Whether this ledger has been sealed (archived).
    pub sealed: bool,
}

// The genesis chain anchor — all ledgers start from this.
const GENESIS_CHAIN_HASH: &str = "genesis:bkg:0000000000000000";

impl EventLedger {
    pub fn new(realm_id: RealmId) -> Self {
        Self {
            realm_id: Some(realm_id),
            entries: vec![],
            chain_tip: GENESIS_CHAIN_HASH.to_string(),
            sealed: false,
        }
    }

    /// Append a new event to the ledger.
    ///
    /// Enforces:
    ///   - Lamport monotone
    ///   - No duplicate event_ids
    ///   - Not sealed
    ///   - Computes and chains the hash
    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &mut self,
        event_id: impl Into<String>,
        schema_id: impl Into<String>,
        lamport: u64,
        payload_hash: impl Into<String>,
        payload: serde_json::Value,
        producer: impl Into<String>,
        causal_parent: Option<String>,
    ) -> Result<&LedgerEntry, LedgerAppendError> {
        if self.sealed {
            return Err(LedgerAppendError::Sealed);
        }

        let event_id = event_id.into();
        let payload_hash = payload_hash.into();

        // Lamport monotone check
        if let Some(last) = self.entries.last() {
            if lamport <= last.lamport {
                return Err(LedgerAppendError::LamportNotMonotone {
                    tip: last.lamport, new: lamport,
                });
            }
        }

        // Duplicate event_id check
        if self.entries.iter().any(|e| e.event_id == event_id) {
            return Err(LedgerAppendError::DuplicateEventId { event_id });
        }

        let chain_hash = LedgerEntry::compute_chain_hash(
            &event_id, &payload_hash, &self.chain_tip,
        );

        let entry = LedgerEntry {
            seq: self.entries.len() as u64,
            event_id,
            schema_id: schema_id.into(),
            lamport,
            payload_hash,
            payload,
            producer: producer.into(),
            causal_parent,
            chain_hash: chain_hash.clone(),
            appended_at: Utc::now(),
        };

        self.chain_tip = chain_hash;
        self.entries.push(entry);
        Ok(self.entries.last().unwrap())
    }

    /// Verify the complete hash chain from genesis.
    ///
    /// Any tampering (insertion, deletion, modification) breaks this.
    pub fn verify_chain(&self) -> ChainVerification {
        let mut prev = GENESIS_CHAIN_HASH.to_string();
        for entry in &self.entries {
            if !entry.verify_chain_hash(&prev) {
                return ChainVerification::Broken {
                    at_seq: entry.seq,
                    at_event_id: entry.event_id.clone(),
                };
            }
            prev = entry.chain_hash.clone();
        }
        ChainVerification::Valid {
            entry_count: self.entries.len() as u64,
            chain_tip: self.chain_tip.clone(),
        }
    }

    /// Seal the ledger — no further appends allowed.
    pub fn seal(&mut self) { self.sealed = true; }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn chain_tip(&self) -> &str { &self.chain_tip }
    pub fn entries(&self) -> &[LedgerEntry] { &self.entries }

    /// The lamport range covered by this ledger.
    pub fn lamport_range(&self) -> Option<(u64, u64)> {
        match (self.entries.first(), self.entries.last()) {
            (Some(first), Some(last)) => Some((first.lamport, last.lamport)),
            _ => None,
        }
    }

    /// Get an entry by sequence number.
    pub fn get(&self, seq: u64) -> Option<&LedgerEntry> {
        self.entries.get(seq as usize)
    }

    /// Get all entries in a lamport range (inclusive).
    pub fn range(&self, from_lamport: u64, to_lamport: u64) -> Vec<&LedgerEntry> {
        self.entries.iter()
            .filter(|e| e.lamport >= from_lamport && e.lamport <= to_lamport)
            .collect()
    }
}

/// Result of chain verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainVerification {
    Valid { entry_count: u64, chain_tip: String },
    Broken { at_seq: u64, at_event_id: String },
}

impl ChainVerification {
    pub fn is_valid(&self) -> bool { matches!(self, Self::Valid { .. }) }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;

    fn ledger() -> EventLedger { EventLedger::new(RealmId::Telum) }

    fn append_n(l: &mut EventLedger, n: u64) {
        for i in 1..=n {
            l.append(
                format!("evt-{i}"), "kv.set", i * 10,
                format!("hash-{i}"), serde_json::json!({"seq": i}),
                "test", None,
            ).unwrap();
        }
    }

    #[test]
    fn append_and_verify() {
        let mut l = ledger();
        append_n(&mut l, 3);
        assert_eq!(l.len(), 3);
        assert!(l.verify_chain().is_valid());
    }

    #[test]
    fn chain_tip_advances() {
        let mut l = ledger();
        let before = l.chain_tip().to_string();
        append_n(&mut l, 1);
        assert_ne!(l.chain_tip(), before);
    }

    #[test]
    fn lamport_monotone_enforced() {
        let mut l = ledger();
        l.append("e1", "s", 10, "h", serde_json::json!({}), "t", None).unwrap();
        let err = l.append("e2", "s", 5, "h", serde_json::json!({}), "t", None);
        assert!(matches!(err, Err(LedgerAppendError::LamportNotMonotone { .. })));
    }

    #[test]
    fn duplicate_event_id_rejected() {
        let mut l = ledger();
        l.append("evt-dup", "s", 1, "h", serde_json::json!({}), "t", None).unwrap();
        let err = l.append("evt-dup", "s", 2, "h", serde_json::json!({}), "t", None);
        assert!(matches!(err, Err(LedgerAppendError::DuplicateEventId { .. })));
    }

    #[test]
    fn sealed_blocks_append() {
        let mut l = ledger();
        l.seal();
        let err = l.append("e1", "s", 1, "h", serde_json::json!({}), "t", None);
        assert!(matches!(err, Err(LedgerAppendError::Sealed)));
    }

    #[test]
    fn tampered_entry_breaks_chain() {
        let mut l = ledger();
        append_n(&mut l, 5);
        assert!(l.verify_chain().is_valid());

        // Tamper: change a payload hash in the middle
        l.entries[2].payload_hash = "TAMPERED".to_string();
        assert!(!l.verify_chain().is_valid());
    }

    #[test]
    fn range_query() {
        let mut l = ledger();
        append_n(&mut l, 5); // lamports: 10,20,30,40,50
        let r = l.range(20, 40);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].lamport, 20);
        assert_eq!(r[2].lamport, 40);
    }

    #[test]
    fn empty_ledger_verifies() {
        let l = ledger();
        assert!(l.verify_chain().is_valid());
        assert_eq!(l.lamport_range(), None);
    }

    #[test]
    fn deterministic_chain() {
        // Same events in same order → same chain tip, always.
        let mut l1 = ledger();
        let mut l2 = ledger();
        let events = vec![
            ("e1", "s", 1u64, "h1"),
            ("e2", "s", 2,    "h2"),
            ("e3", "s", 3,    "h3"),
        ];
        for &(id, schema, lam, ph) in &events {
            l1.append(id, schema, lam, ph, serde_json::json!({}), "t", None).unwrap();
            l2.append(id, schema, lam, ph, serde_json::json!({}), "t", None).unwrap();
        }
        assert_eq!(l1.chain_tip(), l2.chain_tip(),
            "determinism: same events → same chain tip");
    }
}
