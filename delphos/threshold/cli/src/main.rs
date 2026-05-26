use anyhow::Result;
use clap::{Parser, Subcommand};
mod commands; mod context; mod output;
use commands::{agent, chat, init, isolate, replay, run, status, verify};

/// BKG — Deterministic Multi-Realm Execution System
/// Single source of truth. One module, one location.
#[derive(Parser)]
#[command(name = "bkg", version, about = "BKG — Deterministic Multi-Realm Execution System")]
struct Cli {
    #[arg(long, env = "BKG_DATA_DIR", default_value = "./bkg-data")]
    data_dir: std::path::PathBuf,
    #[arg(long, short, global = true)] verbose: bool,
    #[command(subcommand)] command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise a new BKG topology (genesis + ledger)
    Init(init::InitArgs),
    /// Execute a task with SWD recording
    Run(run::RunArgs),
    /// Verify hash chain and SWD signatures
    Verify(verify::VerifyArgs),
    /// Reconstruct system state from the event ledger
    Replay(replay::ReplayArgs),
    /// Output current system state as JSON
    Status(status::StatusArgs),
    /// Isolate a corrupted branch into the Isolation Layer
    Isolate(isolate::IsolateArgs),
    /// Interactive LLM conversation with /slash command support
    Chat(chat::ChatArgs),
    /// Manage agents in this runtime session
    Agent(agent::AgentArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let level = if cli.verbose { "debug" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| level.into()))
        .with_writer(std::io::stderr).init();
    let ctx = context::BkgContext::new(cli.data_dir);
    match cli.command {
        Commands::Init(a)    => init::run(&ctx, a),
        Commands::Run(a)     => run::run(&ctx, a),
        Commands::Verify(a)  => verify::run(&ctx, a),
        Commands::Replay(a)  => replay::run(&ctx, a),
        Commands::Status(a)  => status::run(&ctx, a),
        Commands::Isolate(a) => isolate::run(&ctx, a),
        Commands::Chat(a)    => chat::run(&ctx, a),
        Commands::Agent(a)   => agent::run(&ctx, a),
    }
}
