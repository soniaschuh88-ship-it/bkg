use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{policy::AccessPolicy, scope::SecretScope};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecretId(pub String);
impl SecretId { pub fn new(name: &str, scope: &SecretScope) -> Self { Self(format!("{}:{}",scope.as_str(),name)) } }

/// A stored secret. Value is encrypted at rest (simulated here with base64).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub id: SecretId, pub name: String, pub scope: SecretScope,
    pub policy: AccessPolicy,
    encrypted_value: String, // base64(value) — in prod: AES-256-GCM
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
    pub read_count: u64,
    pub last_read_by: Option<String>,
}
impl Secret {
    pub fn new(name: impl Into<String>, value: impl AsRef<[u8]>, scope: SecretScope, policy: AccessPolicy) -> Self {
        use std::fmt::Write;
        let enc = value.as_ref().iter().fold(String::new(), |mut s, b| { let _ = write!(s, "{b:02x}"); s });
        let now = Utc::now();
        let name = name.into();
        let id = SecretId::new(&name, &scope);
        Self { id, name, scope, policy, encrypted_value: enc, created_at: now, updated_at: now, read_count: 0, last_read_by: None }
    }
    pub fn decrypt(&mut self, reader: Option<&str>) -> Option<Vec<u8>> {
        if self.policy == AccessPolicy::Deny { return None; }
        self.read_count += 1;
        self.last_read_by = reader.map(String::from);
        // hex decode
        (0..self.encrypted_value.len()).step_by(2)
            .map(|i| u8::from_str_radix(&self.encrypted_value[i..i+2], 16).ok())
            .collect()
    }
    pub fn update_value(&mut self, value: impl AsRef<[u8]>) {
        use std::fmt::Write;
        self.encrypted_value = value.as_ref().iter().fold(String::new(), |mut s, b| { let _ = write!(s, "{b:02x}"); s });
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn create_and_decrypt() {
        let mut s = Secret::new("DB_PASS","secret123",SecretScope::Global,AccessPolicy::Auto);
        let val = s.decrypt(Some("admin")).unwrap();
        assert_eq!(String::from_utf8(val).unwrap(), "secret123");
        assert_eq!(s.read_count, 1);
    }
    #[test] fn deny_returns_none() {
        let mut s = Secret::new("X","val",SecretScope::Global,AccessPolicy::Deny);
        assert!(s.decrypt(None).is_none());
    }
    #[test] fn update_value() {
        let mut s = Secret::new("K","old",SecretScope::Global,AccessPolicy::Auto);
        s.update_value("new");
        let v = s.decrypt(None).unwrap();
        assert_eq!(String::from_utf8(v).unwrap(),"new");
    }
}
