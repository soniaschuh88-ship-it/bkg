use anyhow::Result;use clap::{Parser,Subcommand};
mod commands;mod context;mod output;
use commands::{agent,chat,init,isolate,providers,replay,run,status,verify};
#[derive(Parser)]#[command(name="bkg",version,about="BKG — Deterministic Multi-Realm Execution System")]
struct Cli{#[arg(long,env="BKG_DATA_DIR",default_value="./bkg-data")]data_dir:std::path::PathBuf,#[arg(long,short,global=true)]verbose:bool,#[command(subcommand)]command:Commands}
#[derive(Subcommand)]enum Commands{Init(init::InitArgs),Run(run::RunArgs),Verify(verify::VerifyArgs),Replay(replay::ReplayArgs),Status(status::StatusArgs),Isolate(isolate::IsolateArgs),Chat(chat::ChatArgs),Agent(agent::AgentArgs),Providers(providers::ProvidersArgs)}
fn main()->Result<()>{let cli=Cli::parse();let level=if cli.verbose{"debug"}else{"warn"};tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_|level.into())).with_writer(std::io::stderr).init();let ctx=context::BkgContext::new(cli.data_dir);match cli.command{Commands::Init(a)=>init::run(&ctx,a),Commands::Run(a)=>run::run(&ctx,a),Commands::Verify(a)=>verify::run(&ctx,a),Commands::Replay(a)=>replay::run(&ctx,a),Commands::Status(a)=>status::run(&ctx,a),Commands::Isolate(a)=>isolate::run(&ctx,a),Commands::Chat(a)=>chat::run(&ctx,a),Commands::Agent(a)=>agent::run(&ctx,a),Commands::Providers(a)=>providers::run(&ctx,a)}}
