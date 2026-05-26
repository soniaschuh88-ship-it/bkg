use crate::types::ProviderModelConfig;
pub fn detect_pricing_exposed(m:&[ProviderModelConfig])->bool {
    m.iter().any(|x|x.cost.input>0.0||x.cost.output>0.0)
}
pub fn is_free_model(m:&ProviderModelConfig,all:&[ProviderModelConfig])->bool {
    if detect_pricing_exposed(all) { m.cost.is_free() }
    else { m.id.to_lowercase().contains("free")||m.name.to_lowercase().contains("free") }
}
pub fn apply_free_filter(models:Vec<ProviderModelConfig>,free_only:bool)->Vec<ProviderModelConfig> {
    if !free_only { return models; }
    let snap=models.clone();
    models.into_iter().filter(|m|is_free_model(m,&snap)).collect()
}
#[cfg(test)] mod tests { use super::*;
    fn m(id:&str,c:f64)->ProviderModelConfig{let mut x=ProviderModelConfig::new(id,id,"p");x.cost.input=c;x}
    #[test] fn route_a(){let ms=vec![m("free",0.0),m("paid",1e-6)];assert!(is_free_model(&ms[0],&ms));assert!(!is_free_model(&ms[1],&ms));}
    #[test] fn route_b(){let ms=vec![m("crofai-free",0.0),m("pro",0.0)];assert!(is_free_model(&ms[0],&ms));assert!(!is_free_model(&ms[1],&ms));}
    #[test] fn exposed(){assert!(detect_pricing_exposed(&[m("a",0.0),m("b",1e-6)]));}
}