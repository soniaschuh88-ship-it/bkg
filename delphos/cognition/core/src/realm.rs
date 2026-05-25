use std::fmt;
use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize,PartialOrd,Ord)]
#[serde(rename_all="snake_case")]
pub enum RealmId { Telum, Causa, Styx, Speculum, Mensa, Katoptron, Anamnesis }
impl RealmId {
    pub fn all() -> &'static [RealmId] { &[RealmId::Telum,RealmId::Causa,RealmId::Styx,RealmId::Speculum,RealmId::Mensa,RealmId::Katoptron,RealmId::Anamnesis] }
    pub fn as_str(self) -> &'static str { match self { RealmId::Telum=>"telum",RealmId::Causa=>"causa",RealmId::Styx=>"styx",RealmId::Speculum=>"speculum",RealmId::Mensa=>"mensa",RealmId::Katoptron=>"katoptron",RealmId::Anamnesis=>"anamnesis" } }
}
impl fmt::Display for RealmId { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { f.write_str(self.as_str()) } }
impl std::str::FromStr for RealmId {
    type Err = crate::BkgError;
    fn from_str(s:&str)->Result<Self,Self::Err> { match s.to_lowercase().as_str() { "telum"=>Ok(RealmId::Telum),"causa"=>Ok(RealmId::Causa),"styx"=>Ok(RealmId::Styx),"speculum"=>Ok(RealmId::Speculum),"mensa"=>Ok(RealmId::Mensa),"katoptron"=>Ok(RealmId::Katoptron),"anamnesis"=>Ok(RealmId::Anamnesis),o=>Err(crate::BkgError::Internal(format!("unknown realm: {o}"))) } }
}
#[cfg(test)] mod tests { use super::*; #[test] fn roundtrip() { for r in RealmId::all() { assert_eq!(r.to_string().parse::<RealmId>().unwrap(),*r); } } }
