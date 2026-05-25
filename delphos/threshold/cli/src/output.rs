use anyhow::Result;
pub fn print_json(v:&impl serde::Serialize)->Result<()>{println!("{}",serde_json::to_string_pretty(v)?);Ok(())}
pub fn print_ok(data:impl serde::Serialize)->Result<()>{print_json(&serde_json::json!({"ok":true,"data":data}))}
