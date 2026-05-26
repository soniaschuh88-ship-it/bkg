use crate::graph::LineageGraph;
pub fn common_ancestor(g: &LineageGraph, a: &str, b: &str) -> Option<String> {
    let aa: std::collections::BTreeSet<String> = g.ancestors_of(a).into_iter().collect();
    g.ancestors_of(b).into_iter().find(|anc| aa.contains(anc))
}
