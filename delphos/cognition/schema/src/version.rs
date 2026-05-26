// version.rs — schema versioning utilities.
// SchemaVersion is defined in schema.rs; this module handles comparison helpers.
// Single source of truth for all schema version logic.
use crate::schema::SchemaVersion;

/// Check if `old` can be safely read by code expecting `new`.
pub fn is_forward_compatible(old: SchemaVersion, new: SchemaVersion) -> bool {
    old.major == new.major && old.minor <= new.minor
}

/// Check if `new` can safely read data written by `old` (backward compat).
pub fn is_backward_compatible(new: SchemaVersion, old: SchemaVersion) -> bool {
    new.major == old.major
}

/// The latest known schema version for a given schema_id.
/// In a full impl this queries the EventSchemaRegistry.
pub fn latest_known() -> SchemaVersion { SchemaVersion::V1 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn forward_compat() { assert!(is_forward_compatible(SchemaVersion::new(1,0), SchemaVersion::new(1,5))); }
    #[test] fn not_forward_compat() { assert!(!is_forward_compatible(SchemaVersion::new(1,5), SchemaVersion::new(1,0))); }
    #[test] fn backward_compat_same_major() { assert!(is_backward_compatible(SchemaVersion::new(1,2), SchemaVersion::new(1,0))); }
    #[test] fn not_backward_compat_diff_major() { assert!(!is_backward_compatible(SchemaVersion::new(2,0), SchemaVersion::new(1,0))); }
}