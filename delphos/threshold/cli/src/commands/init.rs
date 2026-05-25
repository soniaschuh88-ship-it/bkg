use anyhow::{bail,Result};
use clap::Args;
use bkg_core::ExecutionSeed;
use bkg_event::FileLedger;
use bkg_kernel::Genesis;
use crate::{context::BkgContext,output::print_ok};
#[derive(Args)]
pub struct InitArgs{#[arg(long)]seed:Option<String>}
pub fn run(ctx:&BkgContext,args:InitArgs)->Result<()>{
    if ctx.is_initialised(){bail!("already initialised at {:?}",ctx.data_dir);}
    ctx.ensure_dirs()?;
    let seed=match args.seed{Some(h)=>ExecutionSeed::from_hex(&h).map_err(|e|anyhow::anyhow!("{e}"))?,None=>ExecutionSeed::random()};
    let mut ledger=FileLedger::open(ctx.ledger_path())?;
    let genesis=Genesis::initialise(seed,&mut ledger)?;
    std::fs::write(ctx.genesis_path(),serde_json::to_string_pretty(&genesis)?)?;
    print_ok(serde_json::json!({"genesis_hash":genesis.locked_hash.to_hex(),"seed":seed.to_hex(),"data_dir":ctx.data_dir}))
}
