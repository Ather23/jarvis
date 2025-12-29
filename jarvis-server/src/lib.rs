pub mod handlers;
pub mod server;

// websocket module is no longer needed - we use axum's built-in WebSocket support in server.rs

pub use server::{build_server, run_server, run_with_config, ServerConfig};
