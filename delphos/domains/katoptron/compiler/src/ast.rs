use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum UiNodeKind{Panel,Card,Text,Badge,Button,Column,Row,Spacer,Chart,Table}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UiNode{
    pub id:String, pub kind:UiNodeKind, pub text:Option<String>,
    pub children:Vec<UiNode>, pub props:BTreeMap<String,serde_json::Value>,
}
impl UiNode{
    pub fn new(id:impl Into<String>,kind:UiNodeKind)->Self{Self{id:id.into(),kind,text:None,children:vec![],props:BTreeMap::new()}}
    pub fn with_text(mut self,t:impl Into<String>)->Self{self.text=Some(t.into());self}
    pub fn with_child(mut self,c:UiNode)->Self{self.children.push(c);self}
    pub fn set_prop(&mut self,k:impl Into<String>,v:serde_json::Value){self.props.insert(k.into(),v);}
    pub fn descendant_count(&self)->usize{self.children.iter().map(|c|1+c.descendant_count()).sum()}
}
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct UiAst{pub root:Option<UiNode>,pub realm_id:String,pub state_version:u64}
impl UiAst{
    pub fn new(realm:impl Into<String>,version:u64)->Self{Self{root:None,realm_id:realm.into(),state_version:version}}
    pub fn set_root(&mut self,n:UiNode){self.root=Some(n);}
    pub fn node_count(&self)->usize{self.root.as_ref().map(|r|1+r.descendant_count()).unwrap_or(0)}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn build_tree(){
        let mut ast=UiAst::new("telum",1);
        let panel=UiNode::new("root",UiNodeKind::Panel).with_child(UiNode::new("title",UiNodeKind::Text).with_text("BKG Dashboard"));
        ast.set_root(panel);
        assert_eq!(ast.node_count(),2);
    }
    #[test] fn props(){let mut n=UiNode::new("x",UiNodeKind::Badge);n.set_prop("color",serde_json::json!("teal"));assert!(n.props.contains_key("color"));}
}
