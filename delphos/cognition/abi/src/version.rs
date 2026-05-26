use serde::{Deserialize, Serialize};

/// ABI version for backward compatibility negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct AbiVersion { pub major: u16, pub minor: u16, pub patch: u16 }
impl AbiVersion {
    pub const CURRENT: Self = Self { major: 1, minor: 0, patch: 0 };
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self { Self { major, minor, patch } }
    /// Compatible if same major + other minor ≤ our minor.
    pub fn is_compatible_with(&self, other: &Self) -> AbiCompatibility {
        if self.major != other.major { return AbiCompatibility::Incompatible { reason: format!("major version mismatch: {} vs {}", self.major, other.major) }; }
        if other.minor > self.minor { return AbiCompatibility::Newer { them: *other, us: *self }; }
        AbiCompatibility::Compatible
    }
}
impl std::fmt::Display for AbiVersion { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}.{}.{}", self.major, self.minor, self.patch) } }

/// Optional capability flags for feature negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum AbiCapability { EventTyping, CausalOrdering, SchemaRegistry, GcSupport, MeshSync, PluginSlots, PhysicsLayout, BqlQuery }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum AbiCompatibility { Compatible, Newer { them: AbiVersion, us: AbiVersion }, Incompatible { reason: String } }
impl AbiCompatibility { pub fn is_ok(&self) -> bool { matches!(self, Self::Compatible | Self::Newer { .. }) } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn compatible_same_major() { let v=AbiVersion::CURRENT; assert!(v.is_compatible_with(&v).is_ok()); }
    #[test] fn incompatible_different_major() { let a=AbiVersion::new(1,0,0); let b=AbiVersion::new(2,0,0); assert!(!a.is_compatible_with(&b).is_ok()); }
    #[test] fn newer_minor() { let us=AbiVersion::new(1,0,0); let them=AbiVersion::new(1,1,0); assert!(us.is_compatible_with(&them).is_ok()); }
    #[test] fn display() { assert_eq!(AbiVersion::CURRENT.to_string(), "1.0.0"); }
}