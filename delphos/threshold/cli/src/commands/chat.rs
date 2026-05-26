// bkg chat — interactive LLM conversation with /slash command support.
// Providers: ANTHROPIC_API_KEY → Anthropic | OPENAI_API_KEY → OpenAI-compat | default → Ollama
// Slash: /help /status /verify /replay /model /system /clear /history /permission /export /quit
use std::io::{self, BufRead, Write};
use anyhow::{bail, Result};
use clap::Args;
use serde_json::json;
use bkg_event::{EventLedger, FileLedger};
use bkg_kernel::Genesis;
use bkg_replay::ReplayEngine;
use bkg_verifier::{
    verify_hash_chain, EnforcementResult, PermissionEnforcer, PermissionMode, PermissionRequest,
};
use crate::{context::BkgContext, output::print_ok};

#[derive(Args)]
pub struct ChatArgs {
    /// LLM model (auto-detected from env if omitted)
    #[arg(long)] model: Option<String>,
    /// Initial system prompt
    #[arg(long)] system_prompt: Option<String>,
    /// Load a saved session (.jsonl)
    #[arg(long)] session: Option<std::path::PathBuf>,
    /// Permission mode: read-only | workspace-write | danger-full-access
    #[arg(long, default_value = "workspace-write")] permission: String,
    /// Non-interactive: send one prompt and exit
    #[arg(long)] prompt: Option<String>,
}

#[derive(Debug, Clone)]
enum Provider {
    Anthropic { api_key: String, model: String },
    OpenAiCompat { api_key: String, base_url: String, model: String },
    Ollama { base_url: String, model: String },
}
impl Provider {
    fn detect(mo: Option<&str>) -> Self {
        if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
            if !k.is_empty() { return Self::Anthropic { api_key: k, model: mo.unwrap_or("claude-3-5-haiku-20241022").into() }; }
        }
        if let Ok(k) = std::env::var("OPENAI_API_KEY") {
            if !k.is_empty() {
                let base = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".into());
                return Self::OpenAiCompat { api_key: k, base_url: base, model: mo.unwrap_or("gpt-4o-mini").into() };
            }
        }
        Self::Ollama { base_url: std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".into()), model: mo.unwrap_or("llama3").into() }
    }
    fn display(&self) -> String {
        match self { Self::Anthropic{model,..}=>format!("anthropic/{model}"), Self::OpenAiCompat{model,..}=>format!("openai/{model}"), Self::Ollama{model,..}=>format!("ollama/{model}") }
    }
    fn model_name(&self) -> &str {
        match self { Self::Anthropic{model,..}|Self::OpenAiCompat{model,..}|Self::Ollama{model,..}=>model }
    }
    fn set_model(&mut self, m: &str) {
        match self { Self::Anthropic{model,..}|Self::OpenAiCompat{model,..}|Self::Ollama{model,..}=>*model=m.to_string() }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Message {
    role: String,
    content: String,
    #[serde(skip_serializing_if="Option::is_none")] timestamp: Option<String>,
}
impl Message {
    fn user(c: impl Into<String>) -> Self { Self{role:"user".into(),content:c.into(),timestamp:Some(chrono::Utc::now().to_rfc3339())} }
    fn assistant(c: impl Into<String>) -> Self { Self{role:"assistant".into(),content:c.into(),timestamp:Some(chrono::Utc::now().to_rfc3339())} }
}

async fn call_llm(provider: &Provider, messages: &[Message], sys: Option<&str>, client: &reqwest::Client) -> Result<String> {
    match provider {
        Provider::Anthropic { api_key, model } => {
            let api_msgs: Vec<serde_json::Value> = messages.iter().filter(|m|m.role!="system").map(|m|json!({"role":m.role,"content":m.content})).collect();
            let mut body = json!({"model":model,"max_tokens":4096,"messages":api_msgs});
            let effective = sys.or_else(||messages.iter().find(|m|m.role=="system").map(|m|m.content.as_str()));
            if let Some(s) = effective { body["system"]=json!(s); }
            let resp = client.post("https://api.anthropic.com/v1/messages").header("x-api-key",api_key).header("anthropic-version","2023-06-01").json(&body).send().await?;
            if !resp.status().is_success() { let st=resp.status(); bail!("Anthropic {st}: {}",resp.text().await.unwrap_or_default()); }
            let raw:serde_json::Value=resp.json().await?;
            Ok(raw["content"][0]["text"].as_str().unwrap_or("").to_string())
        }
        Provider::OpenAiCompat { api_key, base_url, model } => {
            let mut api_msgs:Vec<serde_json::Value>=Vec::new();
            if let Some(s)=sys{api_msgs.push(json!({"role":"system","content":s}));}
            api_msgs.extend(messages.iter().filter(|m|m.role!="system").map(|m|json!({"role":m.role,"content":m.content})));
            let body=json!({"model":model,"messages":api_msgs,"max_tokens":4096});
            let resp=client.post(format!("{base_url}/v1/chat/completions")).header("authorization",format!("Bearer {api_key}")).json(&body).send().await?;
            if !resp.status().is_success(){let st=resp.status();bail!("OpenAI {st}: {}",resp.text().await.unwrap_or_default());}
            let raw:serde_json::Value=resp.json().await?;
            Ok(raw["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string())
        }
        Provider::Ollama { base_url, model } => {
            let mut api_msgs:Vec<serde_json::Value>=Vec::new();
            if let Some(s)=sys{api_msgs.push(json!({"role":"system","content":s}));}
            api_msgs.extend(messages.iter().filter(|m|m.role!="system").map(|m|json!({"role":m.role,"content":m.content})));
            let resp=client.post(format!("{base_url}/api/chat")).json(&json!({"model":model,"messages":api_msgs,"stream":false})).send().await?;
            if !resp.status().is_success(){bail!("Ollama {} — run: ollama serve",resp.status());}
            let raw:serde_json::Value=resp.json().await?;
            Ok(raw["message"]["content"].as_str().unwrap_or("").to_string())
        }
    }
}

fn parse_mode(s: &str) -> Option<PermissionMode> {
    match s.to_lowercase().replace('-',"_").as_str() {
        "read_only"|"readonly"|"ro" => Some(PermissionMode::ReadOnly),
        "workspace_write"|"write"|"rw" => Some(PermissionMode::WorkspaceWrite),
        "danger_full_access"|"danger"|"full" => Some(PermissionMode::DangerFullAccess),
        _ => None,
    }
}

enum SlashResult { Reply(String), Clear, Quit, NotSlash }

fn handle_slash(input: &str, provider: &mut Provider, messages: &mut Vec<Message>, sys: &mut Option<String>, permission: &mut PermissionMode, ctx: &BkgContext) -> SlashResult {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') { return SlashResult::NotSlash; }
    let parts: Vec<&str> = trimmed.splitn(2,' ').collect();
    let cmd = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s|s.trim()).unwrap_or("");

    match cmd.as_str() {
        "/help"|"/?" => SlashResult::Reply(format!(
            "BKG Chat — slash commands\n\
             /help                  This help\n\
             /status                BKG system status\n\
             /verify                Hash-chain verification\n\
             /replay                Reconstruct ledger state\n\
             /model [name]          Get or switch model ({})\n\
             /system [text]         Get or set system prompt\n\
             /clear                 Clear history\n\
             /history               Print history\n\
             /permission [mode]     Get or set permission mode\n\
             /export                Save session to JSONL\n\
             /quit                  Exit\n\
             Modes: read-only | workspace-write | danger-full-access\n\
             Provider: {}", provider.model_name(), provider.display())),

        "/status" => {
            if !ctx.is_initialised() { return SlashResult::Reply("Not initialised — run `bkg init` first.".into()); }
            SlashResult::Reply((||->anyhow::Result<String>{
                let genesis:Genesis=serde_json::from_str(&std::fs::read_to_string(ctx.genesis_path())?)?;
                let ledger=FileLedger::open(ctx.ledger_path())?;
                Ok(format!("Genesis : {}…\nEvents  : {}\nVerified: {}",&genesis.locked_hash.to_hex()[..16],ledger.len(),if genesis.verify().is_ok(){"✓"}else{"✗ TAMPERED"}))
            })().unwrap_or_else(|e|format!("Error: {e}")))
        }

        "/verify" => {
            if !ctx.is_initialised(){return SlashResult::Reply("Not initialised.".into());}
            SlashResult::Reply(match FileLedger::open(ctx.ledger_path()){
                Ok(l)=>{let r=verify_hash_chain(&l);if r.is_valid(){format!("✓ Chain valid — {} events.",r.events_verified)}else{format!("✗ BROKEN at {:?} ({} failures)",r.first_broken_index,r.report.failure_count())}}
                Err(e)=>format!("Error: {e}"),
            })
        }

        "/replay" => {
            if !ctx.is_initialised(){return SlashResult::Reply("Not initialised.".into());}
            SlashResult::Reply((||->anyhow::Result<String>{
                let ledger=FileLedger::open(ctx.ledger_path())?;
                let state=ReplayEngine::reconstruct_state(&ledger,None)?;
                let mut out=format!("Events : {}\nHash   : {}…\n",state.event_count(),&state.cumulative_hash.to_hex()[..16]);
                for(realm,payload)in&state.per_realm_state{out.push_str(&format!("Realm {realm:12}: {}\n",serde_json::to_string(payload).unwrap_or_default()));}
                Ok(out)
            })().unwrap_or_else(|e|format!("Replay error: {e}")))
        }

        "/model" => {
            if arg.is_empty(){SlashResult::Reply(format!("Current model: {}",provider.display()))}
            else{provider.set_model(arg);SlashResult::Reply(format!("Model → {}",provider.display()))}
        }

        "/system" => {
            if arg.is_empty(){SlashResult::Reply(sys.as_deref().map(|s|format!("System prompt:\n{s}")).unwrap_or_else(||"No system prompt set.".into()))}
            else{*sys=Some(arg.to_string());SlashResult::Reply(format!("System prompt updated ({} chars).",arg.len()))}
        }

        "/clear" => { messages.clear(); SlashResult::Clear }

        "/history" => {
            if messages.is_empty(){return SlashResult::Reply("No history yet.".into());}
            SlashResult::Reply(messages.iter().map(|m|format!("[{}] {}",m.role.to_uppercase(),m.content.chars().take(100).collect::<String>())).collect::<Vec<_>>().join("\n──\n"))
        }

        "/permission"|"/permissions" => {
            if arg.is_empty(){
                let bash=PermissionEnforcer::new().check(&PermissionRequest::new("bash","{}",*permission));
                SlashResult::Reply(format!("Mode   : {}\nBash   : {}\nWrites : {}\nFull   : {}",permission.as_str(),if bash.is_allowed(){"allowed"}else{"denied"},permission.allows_write(),permission.allows_full_access()))
            } else {
                match parse_mode(arg){
                    Some(m)=>{*permission=m;SlashResult::Reply(format!("Permission → {}",m.as_str()))}
                    None=>SlashResult::Reply(format!("Unknown mode '{arg}'. Use: read-only | workspace-write | danger-full-access"))
                }
            }
        }

        "/export" => {
            let path=ctx.data_dir.join(format!("session_{}.jsonl",chrono::Utc::now().format("%Y%m%d_%H%M%S")));
            SlashResult::Reply(match(||->anyhow::Result<()>{
                std::fs::create_dir_all(&ctx.data_dir)?;
                let mut f=std::fs::File::create(&path)?;
                for msg in messages.iter(){writeln!(f,"{}",serde_json::to_string(msg)?)?;}
                Ok(())
            })(){Ok(_)=>format!("Exported to {}",path.display()),Err(e)=>format!("Export failed: {e}")})
        }

        "/quit"|"/exit"|"/q" => SlashResult::Quit,
        _ => SlashResult::Reply(format!("Unknown command: {cmd}\nType /help for available commands.")),
    }
}

pub fn run(ctx: &BkgContext, args: ChatArgs) -> Result<()> {
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(run_async(ctx, args))
}

async fn run_async(ctx: &BkgContext, args: ChatArgs) -> Result<()> {
    let mut provider = Provider::detect(args.model.as_deref());
    let mut sys: Option<String> = args.system_prompt.clone();
    let mut messages: Vec<Message> = Vec::new();
    let mut permission = parse_mode(&args.permission).unwrap_or(PermissionMode::WorkspaceWrite);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build()?;

    if let Some(ref path) = args.session {
        if path.exists() {
            for line in std::io::BufReader::new(std::fs::File::open(path)?).lines() {
                let line = line?;
                if let Ok(msg) = serde_json::from_str::<Message>(&line) { messages.push(msg); }
            }
            eprintln!("Loaded {} messages from session.", messages.len());
        }
    }

    if let Some(prompt) = args.prompt {
        messages.push(Message::user(&prompt));
        let reply = call_llm(&provider, &messages, sys.as_deref(), &client).await
            .map_err(|e| anyhow::anyhow!("LLM error: {e}"))?;
        return print_ok(json!({"reply":reply,"model":provider.display()}));
    }

    eprintln!("BKG Chat  ·  {}  ·  permission: {}", provider.display(), permission.as_str());
    eprintln!("Type /help for slash commands, /quit to exit.");
    eprintln!("─────────────────────────────────────────────");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("you> "); stdout.flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 { break; }
        let input = line.trim();
        if input.is_empty() { continue; }

        match handle_slash(input, &mut provider, &mut messages, &mut sys, &mut permission, ctx) {
            SlashResult::Reply(r) => { println!("bkg> {r}\n"); continue; }
            SlashResult::Clear    => { println!("bkg> Conversation cleared.\n"); continue; }
            SlashResult::Quit     => { println!("bkg> Goodbye."); break; }
            SlashResult::NotSlash => {}
        }

        let lc = input.to_lowercase();
        if lc.contains("rm -rf") || lc.contains("format disk") {
            let check = PermissionEnforcer::new().check(
                &PermissionRequest::new("bash", format!("{{\"command\":\"{input}\"}}"), permission));
            if let EnforcementResult::Deny { reason } = check {
                println!("bkg> [Permission denied] {reason}\n"); continue;
            }
        }

        messages.push(Message::user(input));
        print!("bkg> "); stdout.flush()?;
        match call_llm(&provider, &messages, sys.as_deref(), &client).await {
            Ok(reply) => { println!("{reply}\n"); messages.push(Message::assistant(&reply)); }
            Err(e)    => { eprintln!("[error] {e}"); messages.pop(); }
        }
    }
    Ok(())
}
