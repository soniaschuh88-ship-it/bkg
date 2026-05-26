use crate::graph::WorldGraph;
pub fn entities_of_type<'a>(g: &'a WorldGraph, t: &str) -> Vec<&'a str> { g.entities_of_type(t) }
