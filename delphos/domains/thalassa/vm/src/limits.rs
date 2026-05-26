use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ResourceLimits{
    pub max_memory_mb:u64, pub max_cpu_percent:f64,
    pub max_time_secs:u64, pub max_open_files:u32, pub max_processes:u32,
}
impl Default for ResourceLimits{fn default()->Self{Self{max_memory_mb:512,max_cpu_percent:50.0,max_time_secs:300,max_open_files:256,max_processes:16}}}
impl ResourceLimits{
    pub fn strict()->Self{Self{max_memory_mb:128,max_cpu_percent:25.0,max_time_secs:60,max_open_files:64,max_processes:4}}
    pub fn permissive()->Self{Self{max_memory_mb:4096,max_cpu_percent:100.0,max_time_secs:3600,max_open_files:1024,max_processes:64}}
    pub fn is_within_memory(&self,used_mb:u64)->bool{used_mb<=self.max_memory_mb}
    pub fn is_within_time(&self,elapsed_secs:u64)->bool{elapsed_secs<=self.max_time_secs}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn default_limits(){let l=ResourceLimits::default();assert!(l.is_within_memory(256));assert!(!l.is_within_memory(1024));}
    #[test] fn strict(){let l=ResourceLimits::strict();assert!(l.is_within_time(59));assert!(!l.is_within_time(61));}
}
