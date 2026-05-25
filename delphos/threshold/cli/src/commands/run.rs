use anyhow::{bail,Result};
use clap::Args;
use bkg_core::{Capability,ExecutionSeed};
use bkg_runtime::{AgentRuntime,TaskPayload};
use crate::{context::BkgContext,output::print_ok};
#[derive(Args)]
pub struct RunArgs{#[arg(long,default_value="task")]label:String,#[arg(long,default_value=r#"{"action":"noop"}"#)]input:String,#[arg(long)]seed:Option<String>}
pub fn run(ctx:&BkgContext,args:RunArgs)->Result<()>{
    if!ctx.is_initialised(){bail!("not initialised — run `bkg init` first");}
    let seed=match args.seed{Some(h)=>ExecutionSeed::from_hex(&h).map_err(|e|anyhow::anyhow!("{e}"))?,None=>ExecutionSeed::random()};
    let input:serde_json::Value=serde_json::from_str(&args.input).map_err(|e|anyhow::anyhow!("bad --input: {e}"))?;
    let mut rt=AgentRuntime::new();
    let agent=rt.spawn("cli-agent",vec![Capability::RuntimeExecute,Capability::CapsuleWrite],None)?;
    let payload=TaskPayload::new(&args.label,input).with_seed(seed);
    let result=rt.execute(agent,payload,None)?;
    print_ok(serde_json::json!({"session_id":result.session_id,"agent_id":result.agent_id,"outcome":format!("{:?}",result.outcome).to_lowercase(),"output":result.output,"ticks_used":result.ticks_used}))
}
