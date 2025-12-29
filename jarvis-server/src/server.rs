use crate::handlers::{chat, chat_json, health_check, models, ServerState};
use jarvis_core::{JarvisAgent, OpenRouterConfig, StreamEvent as JarvisStreamEvent};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

impl ServerConfig {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn address(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8080)))
    }
}

pub async fn build_server(agent: JarvisAgent, config: Option<ServerConfig>) -> (Router, ServerConfig) {
    let config = config.unwrap_or_default();
    let state = ServerState::new(agent);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/chat", post(chat))
        .route("/api/chat/json", post(chat_json))
        .route("/api/models", get(models))
        .route("/ws", get(websocket_handler))
        .with_state(state);

    (app, config)
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_axum_websocket(socket, Arc::new(state)))
}

async fn handle_axum_websocket(socket: WebSocket, state: Arc<ServerState>) {
    use futures::SinkExt;

    let (mut sender, mut receiver) = socket.split();

    while let Some(message) = receiver.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let request: serde_json::Result<serde_json::Value> =
                    serde_json::from_str(text.as_str());

                match request {
                    Ok(json) => {
                        let prompt = json
                            .get("prompt")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        if prompt.is_empty() {
                            let _ = sender
                                .send(Message::Text(
                                    r#"{"error":"no prompt provided"}"#.into(),
                                ))
                                .await;
                            continue;
                        }

                        // Get the stream from the agent
                        let stream_result = {
                            let mut agent_guard = state.agent.lock().await;
                            agent_guard.run_stream(&prompt).await
                        };

                        match stream_result {
                            Ok(mut stream) => {
                                // Process stream events
                                while let Some(event) = stream.next().await {
                                    let msg: String = match event {
                                        Ok(JarvisStreamEvent::Text(text)) => {
                                            format!(
                                                r#"{{"type":"text","content":"{}"}}"#,
                                                text.escape_default()
                                            )
                                        }
                                        Ok(JarvisStreamEvent::Reasoning(thought)) => {
                                            format!(
                                                r#"{{"type":"thinking","content":"{}"}}"#,
                                                thought.escape_default()
                                            )
                                        }
                                        Ok(JarvisStreamEvent::ToolCall { name, args }) => {
                                            format!(
                                                r#"{{"type":"tool_call","name":"{}","args":"{}"}}"#,
                                                name, args
                                            )
                                        }
                                        Ok(JarvisStreamEvent::ToolResult { call_id, result }) => {
                                            format!(
                                                r#"{{"type":"tool_result","call_id":"{}","result":"{}"}}"#,
                                                call_id,
                                                result.escape_default()
                                            )
                                        }
                                        Ok(JarvisStreamEvent::Done) => {
                                            r#"{"type":"done"}"#.to_string()
                                        }
                                        Err(e) => {
                                            format!(r#"{{"type":"error","content":"{}"}}"#, e)
                                        }
                                    };

                                    if sender.send(Message::Text(msg.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = sender
                                    .send(Message::Text(
                                        format!(r#"{{"type":"error","content":"{}"}}"#, e).into(),
                                    ))
                                    .await;
                            }
                        }
                    }
                    Err(_) => {
                        let _ = sender
                            .send(Message::Text(r#"{"error":"invalid json"}"#.into()))
                            .await;
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

pub async fn run_server(agent: JarvisAgent, config: Option<ServerConfig>) -> Result<(), anyhow::Error> {
    let (app, config) = build_server(agent, config).await;
    let addr = config.address();

    tracing::info!("Starting Jarvis server on http://{}", addr);
    tracing::info!("Endpoints:");
    tracing::info!("  GET  /health          - Health check");
    tracing::info!("  POST /api/chat        - Streaming chat (SSE)");
    tracing::info!("  POST /api/chat/json   - Non-streaming JSON chat");
    tracing::info!("  GET  /api/models      - List available models");
    tracing::info!("  WS   /ws              - WebSocket streaming");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn run_with_config(
    config: OpenRouterConfig,
    server_config: Option<ServerConfig>,
    enable_shell: bool,
) -> Result<(), anyhow::Error> {
    let agent = if enable_shell {
        JarvisAgent::new(config)?.with_shell_tool()
    } else {
        JarvisAgent::new(config)?
    };

    run_server(agent, server_config).await
}
