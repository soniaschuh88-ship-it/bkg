use bkg_core::Hash256;
pub fn hash_bytes(d:&[u8])->Hash256{Hash256(*blake3::hash(d).as_bytes())}
pub fn hash_concatenated(parts:&[&[u8]])->Hash256{let mut h=blake3::Hasher::new();for p in parts{h.update(p);}Hash256(*h.finalize().as_bytes())}
pub fn hash_event_fields(id:&[u8],realm:&str,ts:u64,payload:&[u8],seed:&[u8],parent:&[u8])->Hash256{
    let mut h=blake3::Hasher::new();
    for f in &[id,realm.as_bytes(),&ts.to_le_bytes()[..],payload,seed,parent]{h.update(&(f.len() as u64).to_le_bytes());h.update(f);}
    Hash256(*h.finalize().as_bytes())
}
pub fn hash_capsule(id:&[u8],v:u64,state:&[u8],prev:&Hash256)->Hash256{hash_concatenated(&[id,&v.to_le_bytes(),state,prev.as_bytes()])}
pub fn hash_swd_root(sid:&[u8],ops:&[Vec<u8>])->Hash256{let mut h=blake3::Hasher::new();h.update(sid);for op in ops{h.update(&(op.len() as u64).to_le_bytes());h.update(op);}Hash256(*h.finalize().as_bytes())}
#[cfg(test)]mod tests{use super::*;
    #[test]fn det(){assert_eq!(hash_bytes(b"x"),hash_bytes(b"x"));}
    #[test]fn concat(){assert_eq!(hash_bytes(b"ab"),hash_concatenated(&[b"a",b"b"]));}
}
