pub mod http_handlers;

pub use http_handlers::{health_check, chat, chat_json, models};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatRequest {
    pub prompt: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_stream")]
    pub stream: bool,
    #[serde(default)]
    pub temperature: Option<f32>,
}

fn default_stream() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCallInfo {
    pub name: String,
    pub args: String,
}

#[derive(Clone)]
pub struct ServerState {
    pub agent: Arc<Mutex<jarvis_core::JarvisAgent>>,
}

impl ServerState {
    pub fn new(agent: jarvis_core::JarvisAgent) -> Self {
        Self {
            agent: Arc::new(Mutex::new(agent)),
        }
    }
}
