use crate::graph::WorldGraph;
pub fn focus_score(g: &WorldGraph, id: &str) -> f64 { (g.relations_from(id).len() + g.edges.iter().filter(|e| e.to == id).count() * 2) as f64 }
