use crate::types::ProviderModelConfig;
static CI:&[(&str,f32)]=&[
    ("claude-opus-4",72.5),("claude-sonnet-4",68.0),("claude-3-5-sonnet",64.0),
    ("claude-3-5-haiku",45.2),("gpt-4o",57.4),("gpt-4o-mini",40.0),
    ("o4-mini",68.2),("o3",75.0),("gemini-2.5-pro",72.8),("gemini-2.5-flash",55.0),
    ("deepseek-r1",72.6),("deepseek-v3",49.2),("llama-3.3-70b",43.0),
    ("llama-3.1-405b",51.0),("qwen2.5-coder-32b",52.7),("codestral",43.0),("phi-4",38.0),
];
pub fn ci_score(id:&str)->Option<f32>{let lo=id.to_lowercase();CI.iter().find(|(f,_)|lo.contains(f)).map(|(_,s)|*s)}
pub fn enhance_name(id:&str,display:&str)->String{match ci_score(id){Some(s)=>format!("{display} (CI: {s:.1})"),None=>display.to_string()}}
pub fn enhance_all(models:Vec<ProviderModelConfig>)->Vec<ProviderModelConfig>{models.into_iter().map(|mut m|{m.name=enhance_name(&m.id,&m.name);m}).collect()}
#[cfg(test)] mod tests { use super::*;
    #[test] fn known(){assert_eq!(ci_score("claude-3-5-sonnet-20241022"),Some(64.0));}
    #[test] fn enhance(){assert!(enhance_name("deepseek-r1","DeepSeek R1").contains("CI:"));}
}