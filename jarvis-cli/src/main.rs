mod cli;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Only init tracing if stderr is a terminal (avoid noise in pipes)
    if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
        tracing_subscriber::fmt::init();
    }

    let args = Cli::parse();

    // Handle -i / --interactive flag (takes precedence)
    if args.interactive {
        cli::commands::handle_interactive(args.model).await?;
        return Ok(());
    }

    // Handle subcommands
    match args.command {
        Some(Commands::Chat { prompt, no_thinking }) => {
            cli::commands::handle_chat(prompt, args.model, no_thinking, args.raw).await?;
        }
        Some(Commands::Run { prompt, json }) => {
            cli::commands::handle_run(prompt, args.model, json, args.raw).await?;
        }
        Some(Commands::Interactive) => {
            cli::commands::handle_interactive(args.model).await?;
        }
        None => {
            // No command given, show help
            println!("{}", Cli::command().render_help());
        }
    }

    Ok(())
}
