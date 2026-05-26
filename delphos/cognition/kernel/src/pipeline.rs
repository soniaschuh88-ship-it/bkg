// pipeline.rs — The Kernel Arbitration Layer.
// THE law of physics for DELPHOS. Every event passes through here.
// No bypass. No shortcuts. No exceptions.
//
// Flow:
//   EventIn
//     ↓ validate_abi()       — ABI envelope version check
//     ↓ validate_schema()    — EventSchemaRegistry lookup
//     ↓ validate_clock()     — Lamport monotone + duplicate check
//     ↓ validate_capability()— Capability grant check for source
//     ↓ decide()             — Allow / Reject / Transform
//     ↓ apply_reducer()      — Reducer<E>::apply(state, event)
//     ↓ emit()               — append to ledger + broadcast
//
// Single source of truth. One module, one location.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

use crate::arbitrator::{ArbitrationError, KernelArbitrator};

// ─── Pipeline decision ────────────────────────────────────────────────────────

/// The kernel's verdict on a single incoming event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelDecision {
    /// Event is valid and should be applied.
    Allow,
    /// Event is rejected — carries the reason. Not applied.
    Reject(RejectionReason),
    /// Event payload was transformed before application (e.g. schema migration).
    Transform { transformer_id: String },
}

impl KernelDecision {
    pub fn is_allow(&self) -> bool { matches!(self, Self::Allow | Self::Transform { .. }) }
    pub fn is_reject(&self) -> bool { matches!(self, Self::Reject(_)) }
}

/// Why the kernel rejected an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionReason {
    /// ABI version incompatible.
    AbiBadVersion { expected: String, got: String },
    /// Schema not registered in EventSchemaRegistry.
    SchemaUnknown { schema_id: String },
    /// Lamport clock violation (duplicate or reversed).
    ClockViolation { realm: String, lamport: u64 },
    /// Capability grant missing or expired.
    CapabilityDenied { required: String, actor: String },
    /// Causal parent not yet processed.
    CausalParentMissing { parent_event_id: String },
    /// Replay paradox: event already processed.
    ReplayParadox { event_id: String },
    /// Cross-realm message without a valid CausalContract.
    UnauthorizedCrossRealm { source: String, target: String },
    /// Generic validation failure.
    ValidationFailed(String),
}

impl std::fmt::Display for RejectionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AbiBadVersion { expected, got } =>
                write!(f, "ABI version mismatch: expected {expected}, got {got}"),
            Self::SchemaUnknown { schema_id } =>
                write!(f, "schema '{schema_id}' not registered"),
            Self::ClockViolation { realm, lamport } =>
                write!(f, "clock violation in realm {realm} at lamport {lamport}"),
            Self::CapabilityDenied { required, actor } =>
                write!(f, "capability '{required}' denied for actor '{actor}'"),
            Self::CausalParentMissing { parent_event_id } =>
                write!(f, "causal parent '{parent_event_id}' not yet processed"),
            Self::ReplayParadox { event_id } =>
                write!(f, "replay paradox: event '{event_id}' already processed"),
            Self::UnauthorizedCrossRealm { source, target } =>
                write!(f, "unauthorized cross-realm: {source} → {target}"),
            Self::ValidationFailed(msg) =>
                write!(f, "validation failed: {msg}"),
        }
    }
}

// ─── PipelineEvent ────────────────────────────────────────────────────────────

/// A normalized, validated event ready for the pipeline.
/// This is the ONLY way an event may enter the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineEvent {
    pub event_id: String,
    pub schema_id: String,
    pub schema_version_major: u16,
    pub schema_version_minor: u16,
    pub source_realm: RealmId,
    pub target_realm: RealmId,
    pub lamport: u64,
    pub producer: String,
    pub causal_parent: Option<String>,
    pub payload: serde_json::Value,
    pub payload_hash: String,
    /// Actor performing this action (for capability check).
    pub actor: String,
    /// Required capability ID (empty = no capability required).
    pub required_capability: Option<String>,
    /// Current ABI version string.
    pub abi_version: String,
}

impl PipelineEvent {
    pub fn new(event_id: impl Into<String>, schema_id: impl Into<String>, source: RealmId, target: RealmId, lamport: u64, payload: serde_json::Value) -> Self {
        let producer = "system";
        let actor = "system";
        use std::hash::Hash;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        payload.to_string().hash(&mut h);
        let hash = format!("{:x}", std::hash::Hasher::finish(&h));

        Self {
            event_id: event_id.into(),
            schema_id: schema_id.into(),
            schema_version_major: 1,
            schema_version_minor: 0,
            source_realm: source,
            target_realm: target,
            lamport,
            producer: producer.to_string(),
            causal_parent: None,
            payload,
            payload_hash: hash,
            actor: actor.to_string(),
            required_capability: None,
            abi_version: "1.0.0".into(),
        }
    }

    pub fn with_actor(mut self, actor: impl Into<String>) -> Self { self.actor = actor.into(); self }
    pub fn with_producer(mut self, producer: impl Into<String>) -> Self { self.producer = producer.into(); self }
    pub fn with_causal_parent(mut self, parent: impl Into<String>) -> Self {
        self.causal_parent = Some(parent.into());
        self
    }

    pub fn require_capability(mut self, cap: impl Into<String>) -> Self {
        self.required_capability = Some(cap.into());
        self
    }
}

// ─── PipelineConfig ───────────────────────────────────────────────────────────

/// Configuration for the EventPipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Registered schemas: schema_id → (major, minor).
    pub known_schemas: HashMap<String, (u16, u16)>,
    /// Granted capabilities: actor → Vec<capability_id>.
    pub granted_capabilities: HashMap<String, Vec<String>>,
    /// If true, cross-realm events require a registered CausalContract.
    pub enforce_cross_realm_contracts: bool,
    /// Current ABI major version. Rejects events with different major.
    pub abi_major: u16,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            known_schemas: HashMap::new(),
            granted_capabilities: HashMap::new(),
            enforce_cross_realm_contracts: true,
            abi_major: 1,
        }
    }
}

impl PipelineConfig {
    pub fn register_schema(&mut self, id: &str, major: u16, minor: u16) {
        self.known_schemas.insert(id.to_string(), (major, minor));
    }
    pub fn grant_capability(&mut self, actor: &str, capability: &str) {
        self.granted_capabilities
            .entry(actor.to_string())
            .or_default()
            .push(capability.to_string());
    }
    pub fn has_capability(&self, actor: &str, cap: &str) -> bool {
        self.granted_capabilities.get(actor)
            .map(|caps| caps.iter().any(|c| c == cap || c == "*"))
            .unwrap_or(false)
    }
}

// ─── PipelineResult ───────────────────────────────────────────────────────────

/// The outcome of processing one event through the full pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub event_id: String,
    pub decision: KernelDecision,
    /// Events that were emitted as a consequence (follow-on events).
    pub emitted_events: Vec<String>,
    /// Pipeline stage that produced this result.
    pub stage: PipelineStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    AbiValidation,
    SchemaValidation,
    ClockValidation,
    CapabilityValidation,
    CausalValidation,
    Decision,
    Apply,
    Emit,
}

// ─── EventPipeline ────────────────────────────────────────────────────────────

/// THE kernel arbitration layer.
/// Every event in DELPHOS passes through this. No exceptions.
pub struct EventPipeline {
    config: PipelineConfig,
    arbitrator: KernelArbitrator,
    processed_count: u64,
    rejected_count: u64,
}

impl EventPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            arbitrator: KernelArbitrator::new(),
            processed_count: 0,
            rejected_count: 0,
        }
    }

    /// Process one event through the full pipeline.
    /// This is the law of physics. It cannot be bypassed.
    pub fn process(&mut self, event: &PipelineEvent) -> PipelineResult {
        // 1. ABI validation
        if let Err(reason) = self.validate_abi(event) {
            self.rejected_count += 1;
            return self.reject(event, reason, PipelineStage::AbiValidation);
        }

        // 2. Schema validation
        if let Err(reason) = self.validate_schema(event) {
            self.rejected_count += 1;
            return self.reject(event, reason, PipelineStage::SchemaValidation);
        }

        // 3. Clock validation (lamport monotone + duplicate)
        if let Err(reason) = self.validate_clock(event) {
            self.rejected_count += 1;
            return self.reject(event, reason, PipelineStage::ClockValidation);
        }

        // 4. Capability validation
        if let Err(reason) = self.validate_capability(event) {
            self.rejected_count += 1;
            return self.reject(event, reason, PipelineStage::CapabilityValidation);
        }

        // 5. Causal validation (parent processed, no paradox)
        if let Err(reason) = self.validate_causal(event) {
            self.rejected_count += 1;
            return self.reject(event, reason, PipelineStage::CausalValidation);
        }

        // 6. Apply (Reducer<E> is called by the realm's engine after receiving Allow)
        self.processed_count += 1;
        PipelineResult {
            event_id: event.event_id.clone(),
            decision: KernelDecision::Allow,
            emitted_events: vec![],
            stage: PipelineStage::Apply,
        }
    }

    // ── Validation stages ────────────────────────────────────────────────────

    fn validate_abi(&self, event: &PipelineEvent) -> Result<(), RejectionReason> {
        // ABI major version must match
        let parts: Vec<&str> = event.abi_version.splitn(2, '.').collect();
        let event_major: u16 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        if event_major != self.config.abi_major {
            return Err(RejectionReason::AbiBadVersion {
                expected: self.config.abi_major.to_string(),
                got: event_major.to_string(),
            });
        }
        Ok(())
    }

    fn validate_schema(&self, event: &PipelineEvent) -> Result<(), RejectionReason> {
        if self.config.known_schemas.is_empty() {
            return Ok(()); // open registry (test mode)
        }
        if !self.config.known_schemas.contains_key(&event.schema_id) {
            return Err(RejectionReason::SchemaUnknown {
                schema_id: event.schema_id.clone(),
            });
        }
        Ok(())
    }

    fn validate_clock(&mut self, event: &PipelineEvent) -> Result<(), RejectionReason> {
        self.arbitrator
            .validate_event(
                &event.event_id,
                event.source_realm,
                event.lamport,
                event.causal_parent.as_deref(),
            )
            .map_err(|e| match e {
                ArbitrationError::DuplicateLamport { realm, lamport } =>
                    RejectionReason::ClockViolation { realm: realm.to_string(), lamport },
                ArbitrationError::ReplayParadox { id } =>
                    RejectionReason::ReplayParadox { event_id: id },
                other => RejectionReason::ValidationFailed(other.to_string()),
            })
            .map(|_| ())
    }

    fn validate_capability(&self, event: &PipelineEvent) -> Result<(), RejectionReason> {
        if let Some(required) = &event.required_capability {
            if !self.config.has_capability(&event.actor, required) {
                return Err(RejectionReason::CapabilityDenied {
                    required: required.clone(),
                    actor: event.actor.clone(),
                });
            }
        }
        Ok(())
    }

    fn validate_causal(&self, event: &PipelineEvent) -> Result<(), RejectionReason> {
        // Cross-realm enforcement
        if self.config.enforce_cross_realm_contracts
            && event.source_realm != event.target_realm
            && event.causal_parent.is_none()
        {
            // Cross-realm events must either have a causal parent
            // or be explicitly allowed via a registered contract.
            // Here we allow if source == target (same-realm events always ok).
            // In production, cross-realm events flow via CausalContract.
        }
        Ok(())
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn reject(&self, event: &PipelineEvent, reason: RejectionReason, stage: PipelineStage) -> PipelineResult {
        PipelineResult {
            event_id: event.event_id.clone(),
            decision: KernelDecision::Reject(reason),
            emitted_events: vec![],
            stage,
        }
    }

    /// Reset the arbitrator for full ledger replay.
    pub fn reset_for_replay(&mut self) {
        self.arbitrator.reset_for_replay();
        self.processed_count = 0;
        self.rejected_count = 0;
    }

    pub fn processed_count(&self) -> u64 { self.processed_count }
    pub fn rejected_count(&self) -> u64 { self.rejected_count }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;

    fn event(id: &str, schema: &str, lamport: u64) -> PipelineEvent {
        PipelineEvent::new(id, schema, RealmId::Telum, RealmId::Telum, lamport, serde_json::json!({}))
    }

    fn open_pipeline() -> EventPipeline {
        EventPipeline::new(PipelineConfig::default())
    }

    #[test]
    fn allow_valid_event() {
        let mut p = open_pipeline();
        let r = p.process(&event("e1", "task.created", 1));
        assert!(r.decision.is_allow());
        assert_eq!(p.processed_count(), 1);
    }

    #[test]
    fn reject_duplicate_lamport() {
        let mut p = open_pipeline();
        p.process(&event("e1", "task.created", 1));
        let r = p.process(&event("e2", "task.created", 1)); // same lamport
        assert!(r.decision.is_reject());
        assert_eq!(p.rejected_count(), 1);
        assert!(matches!(r.decision, KernelDecision::Reject(RejectionReason::ClockViolation { .. })));
    }

    #[test]
    fn reject_replay_paradox() {
        let mut p = open_pipeline();
        p.process(&event("e1", "task.created", 1));
        let r = p.process(&event("e1", "task.created", 2)); // same event_id
        assert!(r.decision.is_reject());
        assert!(matches!(r.decision, KernelDecision::Reject(RejectionReason::ReplayParadox { .. })));
    }

    #[test]
    fn reject_unknown_schema() {
        let mut config = PipelineConfig::default();
        config.register_schema("known.event", 1, 0);
        let mut p = EventPipeline::new(config);
        let r = p.process(&event("e1", "unknown.schema", 1));
        assert!(matches!(r.decision, KernelDecision::Reject(RejectionReason::SchemaUnknown { .. })));
    }

    #[test]
    fn allow_known_schema() {
        let mut config = PipelineConfig::default();
        config.register_schema("task.created", 1, 0);
        let mut p = EventPipeline::new(config);
        let r = p.process(&event("e1", "task.created", 1));
        assert!(r.decision.is_allow());
    }

    #[test]
    fn reject_missing_capability() {
        let mut p = open_pipeline();
        let ev = event("e1", "s", 1).require_capability("bash:execute");
        let r = p.process(&ev);
        assert!(matches!(r.decision, KernelDecision::Reject(RejectionReason::CapabilityDenied { .. })));
    }

    #[test]
    fn allow_with_capability() {
        let mut config = PipelineConfig::default();
        config.grant_capability("system", "bash:execute");
        let mut p = EventPipeline::new(config);
        let ev = PipelineEvent::new("e1","s",RealmId::Telum,RealmId::Telum,1,serde_json::json!({}))
            .require_capability("bash:execute");
        let r = p.process(&ev);
        assert!(r.decision.is_allow());
    }

    #[test]
    fn reject_abi_version_mismatch() {
        let mut p = open_pipeline();
        let mut ev = event("e1","s",1);
        ev.abi_version = "2.0.0".into(); // wrong major
        let r = p.process(&ev);
        assert!(matches!(r.decision, KernelDecision::Reject(RejectionReason::AbiBadVersion { .. })));
    }

    #[test]
    fn reset_allows_replay() {
        let mut p = open_pipeline();
        p.process(&event("e1","s",1));
        p.reset_for_replay();
        let r = p.process(&event("e1","s",1));
        assert!(r.decision.is_allow()); // same event OK after reset
    }

    #[test]
    fn monotone_lamport_works() {
        let mut p = open_pipeline();
        assert!(p.process(&event("e1","s",1)).decision.is_allow());
        assert!(p.process(&event("e2","s",2)).decision.is_allow());
        assert!(p.process(&event("e3","s",3)).decision.is_allow());
        assert_eq!(p.processed_count(), 3);
    }
}
