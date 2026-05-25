use std::collections::{HashMap,HashSet};
use bkg_core::{BkgError,BkgResult};
#[derive(Debug,Default)]
pub struct SemanticIndex{tag_to_nodes:HashMap<String,HashSet<String>>,node_to_tags:HashMap<String,HashSet<String>>}
impl SemanticIndex{
    pub fn new()->Self{Self::default()}
    pub fn register(&mut self,id:impl Into<String>,tags:impl IntoIterator<Item=impl Into<String>>){let id=id.into();for t in tags{let t:String=t.into();self.tag_to_nodes.entry(t.clone()).or_default().insert(id.clone());self.node_to_tags.entry(id.clone()).or_default().insert(t);}}
    pub fn deregister(&mut self,id:&str){if let Some(tags)=self.node_to_tags.remove(id){for t in tags{if let Some(s)=self.tag_to_nodes.get_mut(&t){s.remove(id);if s.is_empty(){self.tag_to_nodes.remove(&t);}}}}}
    pub fn find_by_tag(&self,tag:&str)->Vec<&str>{self.tag_to_nodes.get(tag).map(|s|s.iter().map(|s|s.as_str()).collect()).unwrap_or_default()}
    pub fn find_any(&self,tags:&[&str])->Vec<String>{let mut r=HashSet::new();for &t in tags{if let Some(s)=self.tag_to_nodes.get(t){r.extend(s.iter().cloned());}}let mut v:Vec<_>=r.into_iter().collect();v.sort();v}
    pub fn find_all(&self,tags:&[&str])->BkgResult<Vec<String>>{if tags.is_empty(){return Err(BkgError::Internal("need ≥1 tag".into()));}let first=self.tag_to_nodes.get(tags[0]).cloned().unwrap_or_default();let r=tags[1..].iter().fold(first,|a,&t|{let o=self.tag_to_nodes.get(t).cloned().unwrap_or_default();a.intersection(&o).cloned().collect()});let mut v:Vec<_>=r.into_iter().collect();v.sort();Ok(v)}
    pub fn tags_for(&self,id:&str)->Vec<&str>{self.node_to_tags.get(id).map(|s|s.iter().map(|s|s.as_str()).collect()).unwrap_or_default()}
}
#[cfg(test)]mod tests{use super::*;
    #[test]fn reg_find(){let mut i=SemanticIndex::new();i.register("n1",["a","b"]);i.register("n2",["a"]);assert_eq!(i.find_by_tag("a").len(),2);}
    #[test]fn find_all(){let mut i=SemanticIndex::new();i.register("n1",["a","b"]);i.register("n2",["a"]);assert_eq!(i.find_all(&["a","b"]).unwrap(),vec!["n1"]);}
    #[test]fn dereg(){let mut i=SemanticIndex::new();i.register("n1",["x"]);i.deregister("n1");assert!(i.find_by_tag("x").is_empty());}
}
