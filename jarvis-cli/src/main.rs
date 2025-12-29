mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Only init tracing if stderr is a terminal (avoid noise in pipes)
    if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
        tracing_subscriber::fmt::init();
    }

    let args = Cli::parse();

    match args.command {
        Commands::Chat {
            prompt,
            model,
            no_thinking,
            raw,
        } => {
            cli::commands::handle_chat(prompt, model, no_thinking, raw).await?;
        }
        Commands::Run {
            prompt,
            model,
            json,
            raw,
        } => {
            cli::commands::handle_run(prompt, model, json, raw).await?;
        }
        Commands::Interactive { model } => {
            cli::commands::handle_interactive(model).await?;
        }
    }

    Ok(())
}
