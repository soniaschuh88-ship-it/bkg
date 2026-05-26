/// System entropy: measures disorder in the current simulation state.
/// Entropy = unresolved dependencies / total nodes.
/// High entropy → system is chaotic. Low entropy → stable.
pub fn system_entropy(total_nodes:usize,blocked_nodes:usize,dependency_edges:usize)->f64{
    if total_nodes==0{return 0.0;}
    let blocked_ratio=blocked_nodes as f64/total_nodes as f64;
    let edge_density=dependency_edges as f64/(total_nodes as f64).max(1.0);
    (blocked_ratio*0.6+edge_density.min(1.0)*0.4).clamp(0.0,1.0)
}
pub fn entropy_band(entropy:f64)->&'static str{
    if entropy<0.2{"stable"}else if entropy<0.5{"moderate"}else if entropy<0.8{"high"}else{"critical"}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn zero_nodes(){assert_eq!(system_entropy(0,0,0),0.0);}
    #[test] fn all_blocked(){assert!(system_entropy(10,10,5)>0.5);}
    #[test] fn bands(){assert_eq!(entropy_band(0.1),"stable");assert_eq!(entropy_band(0.9),"critical");}
}
