use bkg_core::ExecutionSeed;
use crate::hash::hash_concatenated;
pub fn derive_seed(p:&ExecutionSeed,tag:&str)->ExecutionSeed{ExecutionSeed::from_bytes(hash_concatenated(&[p.as_bytes(),tag.as_bytes()]).0)}
pub fn random_seed()->ExecutionSeed{ExecutionSeed::from_bytes(hash_concatenated(&[ExecutionSeed::random().as_bytes(),b"bkg-v1"]).0)}
#[cfg(test)]mod tests{use super::*;
    #[test]fn det(){let p=ExecutionSeed::from_bytes([1u8;32]);assert_eq!(derive_seed(&p,"a"),derive_seed(&p,"a"));}
    #[test]fn diff(){let p=ExecutionSeed::from_bytes([1u8;32]);assert_ne!(derive_seed(&p,"a"),derive_seed(&p,"b"));}
}
