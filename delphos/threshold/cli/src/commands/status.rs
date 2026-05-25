use anyhow::{bail,Result};
use clap::Args;
use bkg_event::{EventLedger,FileLedger};
use bkg_kernel::Genesis;
use crate::{context::BkgContext,output::print_ok};
#[derive(Args)]
pub struct StatusArgs{}
pub fn run(ctx:&BkgContext,_:StatusArgs)->Result<()>{
    if!ctx.is_initialised(){bail!("not initialised — run `bkg init` first");}
    let genesis:Genesis=serde_json::from_str(&std::fs::read_to_string(ctx.genesis_path())?)?;
    let ledger=FileLedger::open(ctx.ledger_path())?;
    print_ok(serde_json::json!({"initialised":true,"genesis":{"hash":genesis.locked_hash.to_hex(),"valid":genesis.verify().is_ok()},"ledger":{"event_count":ledger.len(),"tail_hash":ledger.tail().map(|e|e.hash.to_hex())},"data_dir":ctx.data_dir}))
}
