use clap::{Parser, Subcommand};
use jarvis_core::OpenRouterConfig;
use jarvis_server::run_with_config;

#[derive(Parser)]
#[command(name = "jarvis-server")]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP/WebSocket server
    Start {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Enable shell command tool
        #[arg(long)]
        with_shell: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    match args.command {
        Commands::Start { host, port, with_shell } => {
            let config = OpenRouterConfig::from_env()?;
            let server_config = jarvis_server::ServerConfig::new(host, port);

            println!("** Jarvis Server **");
            println!();
            println!("Starting server on http://{}:{}", server_config.host, server_config.port);
            println!();
            println!("Endpoints:");
            println!("  GET  /health          - Health check");
            println!("  POST /api/chat        - Streaming chat (SSE)");
            println!("  POST /api/chat/json   - Non-streaming JSON chat");
            println!("  GET  /api/models      - List available models");
            println!("  WS   /ws              - WebSocket streaming");
            println!();

            run_with_config(config, Some(server_config), with_shell).await?;
        }
    }

    Ok(())
}
