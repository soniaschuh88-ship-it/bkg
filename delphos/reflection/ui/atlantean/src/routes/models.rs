use std::sync::Arc;
use axum::{extract::{Query,State}, Json};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use bkg_providers::ProviderModelConfig;
use crate::state::AppState;

type S = Arc<RwLock<AppState>>;

#[derive(Deserialize)]
pub struct ModelsQuery { pub mode: Option<String> }

#[derive(Serialize)]
pub struct ModelsResponse { pub models: Vec<ModelDto>, pub mode: String }

#[derive(Serialize)]
pub struct ModelDto {
    pub id: String, pub name: String, pub provider_id: String,
    pub free: bool, pub reasoning: bool, pub context_window: u64,
}

impl From<&ProviderModelConfig> for ModelDto {
    fn from(m: &ProviderModelConfig) -> Self {
        Self { id: m.id.clone(), name: m.name.clone(), provider_id: m.provider_id.clone(),
               free: m.is_free(), reasoning: m.reasoning, context_window: m.context_window }
    }
}

pub async fn list_models(State(s): State<S>, Query(q): Query<ModelsQuery>) -> Json<ModelsResponse> {
    let s = s.read().await;
    let effective_mode = q.mode.as_deref().unwrap_or(s.mode.as_str());

    let models: Vec<ModelDto> = if effective_mode == "private" {
        // Private: Ollama (local) + WebLLM options
        let ollama = s.registry.models_for("ollama").iter().map(|m| ModelDto::from(*m)).collect::<Vec<_>>();
        let webllm = vec![
            ModelDto{id:"webllm:Llama-3.2-1B-Instruct-q4f16_1-MLC".into(),name:"Llama 3.2 1B (WebLLM, browser)".into(),provider_id:"webllm".into(),free:true,reasoning:false,context_window:4096},
            ModelDto{id:"webllm:Phi-3.5-mini-instruct-q4f16_1-MLC".into(),name:"Phi-3.5 Mini (WebLLM, browser)".into(),provider_id:"webllm".into(),free:true,reasoning:false,context_window:4096},
            ModelDto{id:"webllm:gemma-2-2b-it-q4f16_1-MLC".into(),name:"Gemma 2 2B (WebLLM, browser)".into(),provider_id:"webllm".into(),free:true,reasoning:false,context_window:4096},
        ];
        { let mut v = webllm; v.extend(ollama); v }
    } else {
        // Cloud: all free models from registry
        s.registry.all_models().iter()
            .filter(|m| m.is_free() || !s.globals.free_only)
            .map(|m| ModelDto::from(*m))
            .collect()
    };

    Json(ModelsResponse { models, mode: effective_mode.to_string() })
}