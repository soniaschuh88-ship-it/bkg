use std::path::PathBuf;
use anyhow::{Context,Result};
#[derive(Debug,Clone)]
pub struct BkgContext{pub data_dir:PathBuf}
impl BkgContext{
    pub fn new(d:PathBuf)->Self{Self{data_dir:d}}
    pub fn ledger_path(&self)->PathBuf{self.data_dir.join("styx.ndjson")}
    pub fn genesis_path(&self)->PathBuf{self.data_dir.join("genesis.json")}
    pub fn swd_dir(&self)->PathBuf{self.data_dir.join("swd")}
    pub fn is_initialised(&self)->bool{self.genesis_path().exists()&&self.ledger_path().exists()}
    pub fn ensure_dirs(&self)->Result<()>{std::fs::create_dir_all(&self.data_dir).with_context(||format!("create {:?}",self.data_dir))?;std::fs::create_dir_all(self.swd_dir()).context("create swd dir")?;Ok(())}
}
