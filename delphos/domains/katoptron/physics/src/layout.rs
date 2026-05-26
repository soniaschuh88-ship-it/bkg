use std::collections::HashMap;
#[derive(Debug,Clone)]
pub struct LayoutPosition{pub x:f64,pub y:f64}
/// Deterministic force-directed layout using a grid initial placement.
pub fn initial_grid_layout(node_ids:&[&str],spacing:f64)->HashMap<String,LayoutPosition>{
    let cols=(node_ids.len() as f64).sqrt().ceil() as usize;
    node_ids.iter().enumerate().map(|(i,id)|{
        let row=(i/cols) as f64; let col=(i%cols) as f64;
        (id.to_string(),LayoutPosition{x:col*spacing,y:row*spacing})
    }).collect()
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn grid(){let ids=["T-1","T-2","T-3","T-4"];let layout=initial_grid_layout(&ids,100.0);assert_eq!(layout.len(),4);}
}
