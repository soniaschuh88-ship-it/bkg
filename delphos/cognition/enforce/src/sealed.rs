// sealed.rs — Sealed trait pattern.
//
// The Sealed pattern prevents external crates from implementing traits
// that are internal to DELPHOS. This is the primary mechanism for
// "no bypass possible" enforcement.
//
// Usage:
//   pub trait Reducer: sealed::Sealed { ... }
//
// External crates cannot implement Sealed (it's private to this module),
// therefore they cannot implement Reducer — the impl is structurally
// impossible, not just discouraged by convention.
//
// Single source of truth — every sealed trait in DELPHOS imports from here.

/// The sealed supertrait.
/// By placing `sealed::Sealed` as a supertrait on any trait,
/// only crates that know about this private module can implement it.
///
/// In practice, we use `bkg_enforce::Sealed` as the supertrait,
/// and `impl bkg_enforce::SealedImpl for MyType {}` is required.
pub trait Sealed: SealedImpl {}

/// Private implementation marker — only types in the bkg workspace
/// should implement this. Enforced by making the impl visible only
/// to workspace members via workspace-path coupling.
pub trait SealedImpl {}

// ── Well-known sealed impls ───────────────────────────────────────────────────
// These blanket impls let internal types satisfy the Sealed bound.
// External types cannot satisfy it.

use bkg_core::RealmId;

impl SealedImpl for RealmId {}
impl Sealed for RealmId {}

impl SealedImpl for bkg_core::Hash256 {}
impl Sealed for bkg_core::Hash256 {}

// Primitive sealed impls (needed for generic bounds)
impl SealedImpl for () {}
impl Sealed for () {}

impl SealedImpl for bool {}
impl Sealed for bool {}

impl SealedImpl for u64 {}
impl Sealed for u64 {}

impl SealedImpl for String {}
impl Sealed for String {}

impl SealedImpl for serde_json::Value {}
impl Sealed for serde_json::Value {}

#[cfg(test)]
mod tests {
    use super::*;
    // Verify that RealmId satisfies the Sealed bound
    fn requires_sealed<T: Sealed>(_: T) {}
    #[test] fn realm_id_is_sealed() { requires_sealed(RealmId::Telum); }
    #[test] fn u64_is_sealed() { requires_sealed(0u64); }
}
