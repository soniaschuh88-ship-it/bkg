use anyhow::{bail,Result};
use clap::Args;
use std::io::Write;
use bkg_event::{EventLedger,FileLedger};
use bkg_verifier::{verify_hash_chain,VerificationStatus};
use crate::{context::BkgContext,output::print_ok};
#[derive(Args)]
pub struct IsolateArgs{#[arg(long)]force:bool}
pub fn run(ctx:&BkgContext,args:IsolateArgs)->Result<()>{
    if!ctx.is_initialised(){bail!("not initialised");}
    let ledger=FileLedger::open(ctx.ledger_path())?;
    let chain=verify_hash_chain(&ledger);
    let corrupted=chain.report.status==VerificationStatus::Failed||chain.first_broken_index.is_some();
    if!corrupted&&!args.force{return print_ok(serde_json::json!({"isolated":false,"reason":"chain is valid"}));}
    let broken=chain.first_broken_index.unwrap_or(0);
    let all=ledger.all_events();
    let iso_dir=ctx.data_dir.join("isolation");
    std::fs::create_dir_all(&iso_dir)?;
    let path=iso_dir.join(format!("branch_{}.ndjson",chrono::Utc::now().timestamp()));
    let mut f=std::fs::File::create(&path)?;
    for e in &all[broken..]{writeln!(f,"{}",serde_json::to_string(e)?)?;}
    print_ok(serde_json::json!({"isolated":true,"broken_at":broken,"events_isolated":all.len()-broken,"archive":path,"forced":args.force}))
}
