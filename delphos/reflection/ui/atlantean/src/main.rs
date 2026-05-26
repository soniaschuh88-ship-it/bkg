//! BKG Atlantean — cyberpunk/Atlantis dashboard server.
//!
//! Serves the full-stack web UI with:
//! - Private mode: WebLLM (browser), Ollama tunnel, node-llama bridge
//! - Cloud mode: 13 free providers via bkg-providers fallback chain
//! - User provider key store, admin global key store
//! - Onboarding wizard
//! - Rate-limited self-registration
//!
//! Single source of truth. One module, one location.

mod routes;
mod state;
mod users;

use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{Router, routing::{get, post}};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::EnvFilter;

pub use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("bkg_atlantean=info".parse()?))
        .init();

    let port: u16 = std::env::var("BKG_PORT")
        .ok().and_then(|p| p.parse().ok()).unwrap_or(7878);

    let data_dir = std::env::var("BKG_DATA_DIR")
        .unwrap_or_else(|_| dirs_home().join(".bkg").to_string_lossy().to_string());

    let state = Arc::new(RwLock::new(AppState::load(&data_dir)?));

    let app = Router::new()
        // Static UI
        .route("/",             get(routes::ui::index))
        .route("/static/{*file}", get(routes::ui::static_file))
        // Mode
        .route("/api/mode",     get(routes::mode::get_mode).put(routes::mode::set_mode))
        // Models
        .route("/api/models",   get(routes::models::list_models))
        // Providers
        .route("/providers/list",          get(routes::providers::list))
        .route("/providers/{id}/models",   get(routes::providers::provider_models))
        .route("/providers/proxy",         post(routes::providers::proxy))
        // User
        .route("/user/providers",  get(routes::users::get_user_keys).put(routes::users::set_user_keys))
        .route("/user/profile",    get(routes::users::profile))
        .route("/user/onboarded",  post(routes::users::mark_onboarded))
        // Admin
        .route("/admin/globals",           get(routes::admin::get_globals).put(routes::admin::set_globals))
        .route("/admin/globals/providers", post(routes::admin::update_provider_keys))
        // Self-registration (3/hr rate limit)
        .route("/api-keys/self-register",  post(routes::users::self_register))
        // Stats + Telemetry
        .route("/api/stats",     get(routes::stats::stats))
        .route("/api/telemetry", get(routes::stats::telemetry))
        // Ollama tunnel — proxies to localhost:11434
        // Agents
        .route("/agents/list",              get(routes::agents::list_agents))
        .route("/agents/{id}/status",       get(routes::agents::agent_status))
        .route("/agents/{id}/credentials",  post(routes::agents::set_agent_credentials))
        // Sessions (Inspector)
        .route("/sessions",                 get(routes::sessions::list_sessions).post(routes::sessions::create_session))
        .route("/sessions/{id}",            get(routes::sessions::get_session).delete(routes::sessions::destroy_session))
        .route("/sessions/{id}/send",       post(routes::sessions::send_message))
        .route("/sessions/{id}/stream",     get(routes::sessions::stream_session))
        .route("/tunnel/ollama/{*path}", get(routes::tunnel::ollama_tunnel).post(routes::tunnel::ollama_tunnel))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("BKG Atlantean listening on http://{addr}");
    tracing::info!("Philosophy: Single source of truth. One module, one location.");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn dirs_home() -> std::path::PathBuf {
    std::env::var("HOME").map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
}

pub fn random_hex(n: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..n).map(|_| format!("{:x}", rng.gen::<u8>() % 16)).collect()
}
