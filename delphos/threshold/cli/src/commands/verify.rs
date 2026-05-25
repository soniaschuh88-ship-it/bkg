use anyhow::{bail,Result};
use clap::Args;
use bkg_event::FileLedger;
use bkg_verifier::verify_hash_chain;
use crate::{context::BkgContext,output::print_ok};
#[derive(Args)]
pub struct VerifyArgs{}
pub fn run(ctx:&BkgContext,_:VerifyArgs)->Result<()>{
    if!ctx.is_initialised(){bail!("not initialised");}
    let ledger=FileLedger::open(ctx.ledger_path())?;
    let result=verify_hash_chain(&ledger);
    let failures:Vec<_>=result.report.checks.iter().filter(|c|c.status==bkg_verifier::CheckStatus::Failed).map(|c|serde_json::json!({"name":c.name,"detail":c.detail})).collect();
    print_ok(serde_json::json!({"status":format!("{:?}",result.report.status).to_lowercase(),"events_verified":result.events_verified,"failure_count":result.report.failure_count(),"failures":failures}))
}
