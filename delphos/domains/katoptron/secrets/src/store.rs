use std::collections::BTreeMap;
use bkg_core::{BkgError, BkgResult};
use crate::{policy::AccessPolicy, scope::SecretScope, secret::{Secret, SecretId}};

#[derive(Debug, Default)]
pub struct SecretsStore { secrets: BTreeMap<String, Secret> }
impl SecretsStore {
    pub fn new() -> Self { Self::default() }
    pub fn set(&mut self, name: impl Into<String>, value: impl AsRef<[u8]>, scope: SecretScope, policy: AccessPolicy) {
        let name = name.into();
        let key = SecretId::new(&name, &scope).0;
        if let Some(s) = self.secrets.get_mut(&key) { s.update_value(value); }
        else { let s = Secret::new(name, value, scope, policy); self.secrets.insert(key, s); }
    }
    pub fn get(&mut self, name: &str, scope: &SecretScope, reader: Option<&str>) -> BkgResult<Vec<u8>> {
        let key = SecretId::new(name, scope).0;
        let s = self.secrets.get_mut(&key).ok_or_else(|| BkgError::Internal(format!("secret '{name}' not found")))?;
        s.decrypt(reader).ok_or_else(|| BkgError::Internal(format!("access denied for secret '{name}'")))
    }
    pub fn delete(&mut self, name: &str, scope: &SecretScope) -> bool { self.secrets.remove(&SecretId::new(name,scope).0).is_some() }
    pub fn list(&self, scope: &SecretScope) -> Vec<&str> { let _prefix = scope.as_str(); self.secrets.values().filter(|s| s.scope==*scope).map(|s| s.name.as_str()).collect() }
    pub fn count(&self) -> usize { self.secrets.len() }
    pub fn materialize_env(&mut self, scope: &SecretScope) -> BTreeMap<String,String> {
        let keys: Vec<String> = self.secrets.values().filter(|s| s.scope==*scope).map(|s| s.name.clone()).collect();
        let mut env = BTreeMap::new();
        for k in keys { if let Ok(v) = self.get(&k,scope,Some("env-export")) { if let Ok(s)=String::from_utf8(v){env.insert(k,s);} } }
        env
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn set_get_delete() {
        let mut s=SecretsStore::new(); s.set("API_KEY","sk-123",SecretScope::Global,AccessPolicy::Auto);
        assert_eq!(String::from_utf8(s.get("API_KEY",&SecretScope::Global,None).unwrap()).unwrap(),"sk-123");
        assert!(s.delete("API_KEY",&SecretScope::Global));
        assert!(s.get("API_KEY",&SecretScope::Global,None).is_err());
    }
    #[test] fn env_export() {
        let mut s=SecretsStore::new();
        s.set("X","hello",SecretScope::Global,AccessPolicy::Auto);
        s.set("Y","world",SecretScope::Global,AccessPolicy::Auto);
        let env=s.materialize_env(&SecretScope::Global);
        assert_eq!(env.get("X").map(|s|s.as_str()),Some("hello"));
    }
    #[test] fn project_scope_isolated() {
        let mut s=SecretsStore::new();
        s.set("K","v1",SecretScope::project("P-1"),AccessPolicy::Auto);
        s.set("K","v2",SecretScope::project("P-2"),AccessPolicy::Auto);
        assert_eq!(s.count(),2);
        let v=s.get("K",&SecretScope::project("P-1"),None).unwrap();
        assert_eq!(String::from_utf8(v).unwrap(),"v1");
    }
}
