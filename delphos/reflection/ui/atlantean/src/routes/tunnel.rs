//! Tunnel routes: reverse-proxy to local APIs.
//!
//! POST/GET /tunnel/ollama/* → http://localhost:11434/*
//! Enables Private Mode: the browser can call BKG's /tunnel endpoint
//! instead of localhost:11434 directly (no CORS issues).

use std::sync::Arc;
use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::StatusCode,
    response::Response,
};
use tokio::sync::RwLock;
use crate::state::AppState;

type S = Arc<RwLock<AppState>>;

pub async fn ollama_tunnel(
    State(s): State<S>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let s = s.read().await;
    let ollama_host = std::env::var("OLLAMA_HOST")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let target_url = format!("{ollama_host}/{path}");
    drop(s);

    let client = reqwest::Client::new();
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .unwrap_or(reqwest::Method::POST);

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut rb = client.request(method, &target_url);
    rb = rb.header("content-type", "application/json");
    if !body_bytes.is_empty() { rb = rb.body(body_bytes.to_vec()); }

    match rb.send().await {
        Ok(resp) => {
            let status = resp.status();
            let _headers = resp.headers().clone();
            let bytes = resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
            let mut response = Response::new(Body::from(bytes));
            *response.status_mut() = axum::http::StatusCode::from_u16(status.as_u16())
                .unwrap_or(axum::http::StatusCode::OK);
            Ok(response)
        }
        Err(e) => {
            tracing::warn!("Ollama tunnel error for {target_url}: {e}");
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}