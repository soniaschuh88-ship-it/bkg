use crate::{ast::{UiAst,UiNode,UiNodeKind},bytecode::{Bytecode,BytecodeOp}};
pub struct UiCompiler;
impl UiCompiler{
    pub fn new()->Self{Self}
    /// Compile a UiAst into Bytecode. Deterministic: same AST = same Bytecode.
    pub fn compile(&self,ast:&UiAst)->Bytecode{
        let mut bc=Bytecode::new(&ast.realm_id,ast.state_version);
        if let Some(root)=&ast.root{self.compile_node(root,&mut bc);}
        bc
    }
    fn compile_node(&self,node:&UiNode,bc:&mut Bytecode){
        match node.kind{
            UiNodeKind::Panel=>{bc.push(BytecodeOp::BeginPanel{id:node.id.clone()});for c in &node.children{self.compile_node(c,bc);}bc.push(BytecodeOp::EndPanel);}
            UiNodeKind::Text=>{bc.push(BytecodeOp::Text{id:node.id.clone(),value:node.text.clone().unwrap_or_default(),style:"default".into()});}
            UiNodeKind::Badge=>{let color=node.props.get("color").and_then(|v|v.as_str()).unwrap_or("gray").to_string();bc.push(BytecodeOp::Badge{id:node.id.clone(),label:node.text.clone().unwrap_or_default(),color});}
            UiNodeKind::Button=>{bc.push(BytecodeOp::Button{id:node.id.clone(),label:node.text.clone().unwrap_or_default(),action:node.props.get("action").and_then(|v|v.as_str()).unwrap_or("").to_string()});}
            UiNodeKind::Row=>{bc.push(BytecodeOp::BeginRow{id:node.id.clone()});for c in &node.children{self.compile_node(c,bc);}bc.push(BytecodeOp::End);}
            UiNodeKind::Column=>{bc.push(BytecodeOp::BeginColumn{id:node.id.clone()});for c in &node.children{self.compile_node(c,bc);}bc.push(BytecodeOp::End);}
            UiNodeKind::Spacer=>{bc.push(BytecodeOp::Spacer{size:node.props.get("size").and_then(|v|v.as_u64()).unwrap_or(8)as u32});}
            _=>{for c in &node.children{self.compile_node(c,bc);}}
        }
    }
}
#[cfg(test)]
mod tests{use super::*;use crate::ast::{UiAst,UiNode,UiNodeKind};
    #[test] fn compile_panel(){
        let mut ast=UiAst::new("telum",1);
        let p=UiNode::new("r",UiNodeKind::Panel).with_child(UiNode::new("t",UiNodeKind::Text).with_text("hello"));
        ast.set_root(p);
        let bc=UiCompiler::new().compile(&ast);
        assert!(!bc.is_empty());
        assert_eq!(bc.len(),3); // BeginPanel + Text + EndPanel
    }
    #[test] fn deterministic(){
        let mut ast=UiAst::new("t",5);
        ast.set_root(UiNode::new("x",UiNodeKind::Text).with_text("BKG"));
        let bc1=UiCompiler::new().compile(&ast);
        let bc2=UiCompiler::new().compile(&ast);
        assert_eq!(bc1.len(),bc2.len());
    }
}
