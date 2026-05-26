use bkg_compiler::bytecode::{Bytecode,BytecodeOp};
pub fn to_ansi(bc:&Bytecode)->String{
    let mut out=String::new();
    for op in &bc.ops{match op{
        BytecodeOp::Text{value,..}=>{out.push_str(value);out.push('\n');}
        BytecodeOp::Badge{label,color,..}=>{let code=match color.as_str(){"teal"=>"36","gold"=>"33","red"=>"31",_=>"37"};out.push_str(&format!("\x1b[{code}m[{label}]\x1b[0m\n"));}
        BytecodeOp::Button{label,..}=>{out.push_str(&format!("[{label}]"));}
        BytecodeOp::Spacer{size}=>{for _ in 0..*size{out.push(' ');}}
        _=>{}
    }}
    out
}
#[cfg(test)]
mod tests{use super::*;use bkg_compiler::bytecode::{Bytecode,BytecodeOp};
    #[test] fn text(){let mut bc=Bytecode::new("t",1);bc.push(BytecodeOp::Text{id:"x".into(),value:"hello BKG".into(),style:"default".into()});assert!(to_ansi(&bc).contains("hello BKG"));}
}
