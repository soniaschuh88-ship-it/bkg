use anyhow::{bail,Result};
use clap::Args;
use bkg_event::FileLedger;
use bkg_replay::ReplayEngine;
use crate::{context::BkgContext,output::print_ok};
#[derive(Args)]
pub struct ReplayArgs{#[arg(long)]until_event:Option<String>,#[arg(long)]until_ts:Option<u64>}
pub fn run(ctx:&BkgContext,args:ReplayArgs)->Result<()>{
    if!ctx.is_initialised(){bail!("not initialised");}
    let ledger=FileLedger::open(ctx.ledger_path())?;
    let state=if let Some(ts)=args.until_ts{ReplayEngine::reconstruct_until_ts(&ledger,ts)?}
    else if let Some(id_str)=args.until_event{let id:bkg_core::EventId=id_str.parse().map_err(|e|anyhow::anyhow!("{e}"))?;ReplayEngine::reconstruct_state(&ledger,Some(&id))?}
    else{ReplayEngine::reconstruct_state(&ledger,None)?};
    let per_realm:Vec<_>=state.per_realm_state.iter().map(|(r,s)|serde_json::json!({"realm":r,"state":s})).collect();
    print_ok(serde_json::json!({"events_replayed":state.event_count(),"cumulative_hash":state.cumulative_hash.to_hex(),"terminal_event_id":state.terminal_event_id.map(|i|i.to_string()),"seed":state.seed.map(|s|s.to_hex()),"per_realm_state":per_realm}))
}
