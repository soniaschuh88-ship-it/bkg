// bkg agent — manage agents in the current runtime session.
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;
use bkg_core::Capability;
use bkg_runtime::AgentRuntime;
use crate::{context::BkgContext, output::print_ok};

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)] command: AgentCommand,
}
#[derive(Subcommand)]
enum AgentCommand {
    /// List agents in this session (ephemeral)
    List,
    /// Spawn a new agent
    Spawn {
        #[arg(long, default_value = "agent")] name: String,
        /// read-only | workspace-write | danger-full-access
        #[arg(long, default_value = "workspace-write")] permission: String,
    },
    /// Show agent details
    Show { id: String },
}

pub fn run(_ctx: &BkgContext, args: AgentArgs) -> Result<()> {
    match args.command {
        AgentCommand::List => print_ok(json!({"agents":[],"note":"Agent state is per-session. Use `bkg chat` for persistent sessions."}))?,
        AgentCommand::Spawn { name, permission } => {
            let caps: Vec<Capability> = if permission.contains("danger") {
                vec![Capability::RuntimeExecute,Capability::CapsuleWrite,Capability::LedgerWrite,Capability::AgentControl]
            } else if permission.contains("workspace")||permission=="rw" {
                vec![Capability::RuntimeExecute,Capability::CapsuleWrite]
            } else {
                vec![Capability::LedgerRead,Capability::CapsuleRead,Capability::Observe]
            };
            let mut rt = AgentRuntime::new();
            let id = rt.spawn(&name, caps, None)?;
            print_ok(json!({"agent_id":id.to_string(),"name":name,"permission":permission,"status":"idle"}))?;
        }
        AgentCommand::Show { id } => print_ok(json!({"agent_id":id,"note":"Cross-session persistence requires bkg-capsule + bkg-store."}))?,
    }
    Ok(())
}
