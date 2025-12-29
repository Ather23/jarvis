use crate::handlers::{ChatRequest, ServerState, ToolCallInfo};
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::StreamExt;
use jarvis_core::StreamEvent as JarvisStreamEvent;
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn chat(
    State(state): State<ServerState>,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(100);
    let agent = state.agent.clone();
    let prompt = request.prompt.clone();
    let stream_mode = request.stream;

    tokio::spawn(async move {
        let mut agent = agent.lock().await;

        if stream_mode {
            match agent.run_stream(&prompt).await {
                Ok(mut stream) => {
                    while let Some(event) = stream.next().await {
                        let sse_event = match event {
                            Ok(JarvisStreamEvent::Text(text)) => {
                                Event::default().event("text").data(text)
                            }
                            Ok(JarvisStreamEvent::Reasoning(thought)) => {
                                Event::default().event("thinking").data(thought)
                            }
                            Ok(JarvisStreamEvent::ToolCall { name, args }) => {
                                let info = ToolCallInfo { name, args };
                                Event::default()
                                    .event("tool_call")
                                    .data(serde_json::to_string(&info).unwrap_or_default())
                            }
                            Ok(JarvisStreamEvent::ToolResult { call_id, result }) => {
                                Event::default()
                                    .event("tool_result")
                                    .data(format!(
                                        r#"{{"call_id":"{}","result":"{}"}}"#,
                                        call_id, result
                                    ))
                            }
                            Ok(JarvisStreamEvent::Done) => {
                                Event::default().event("done").data(r#"{"done":true}"#)
                            }
                            Err(e) => Event::default()
                                .event("error")
                                .data(format!(r#"{{"error":"{}"}}"#, e)),
                        };
                        if tx.send(Ok(sse_event)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Ok(Event::default()
                            .event("error")
                            .data(format!(r#"{{"error":"{}"}}"#, e))))
                        .await;
                }
            }
        } else {
            match agent.run(&prompt).await {
                Ok(response) => {
                    let _ = tx
                        .send(Ok(Event::default().event("text").data(response)))
                        .await;
                    let _ = tx
                        .send(Ok(Event::default().event("done").data(r#"{"done":true}"#)))
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(Ok(Event::default()
                            .event("error")
                            .data(format!(r#"{{"error":"{}"}}"#, e))))
                        .await;
                }
            }
        }
    });

    Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default())
}

pub async fn chat_json(
    State(state): State<ServerState>,
    Json(request): Json<ChatRequest>,
) -> Json<serde_json::Value> {
    let mut agent = state.agent.lock().await;
    let response = agent.run(&request.prompt).await;

    match response {
        Ok(response) => Json(serde_json::json!({
            "response": response,
            "done": true
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
            "done": false
        })),
    }
}

pub async fn models() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "models": [
            "google/gemini-2.5-flash",
            "google/gemini-2.5-pro",
            "anthropic/claude-3-5-sonnet",
            "openai/gpt-4o"
        ],
        "default": "google/gemini-2.5-flash"
    }))
}
