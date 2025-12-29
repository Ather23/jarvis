use clap::{Parser, Subcommand};
use futures::StreamExt;
use jarvis_core::{JarvisAgent, OpenRouterConfig, StreamEvent};
use std::io::{self, BufRead, IsTerminal, Write};

#[derive(Parser)]
#[command(name = "jarvis")]
#[command(author, version, about = "Jarvis AI Agent CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Chat with streaming response
    Chat {
        /// The prompt to send (use "-" or omit to read from stdin)
        #[arg(default_value = "-")]
        prompt: String,
        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
        /// Disable thinking output
        #[arg(long)]
        no_thinking: bool,
        /// Raw output (no decorations, suitable for piping)
        #[arg(long, short)]
        raw: bool,
    },
    /// Non-streaming response
    Run {
        /// The prompt to send (use "-" or omit to read from stdin)
        #[arg(default_value = "-")]
        prompt: String,
        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Raw output (no decorations, suitable for piping)
        #[arg(long, short)]
        raw: bool,
    },
    /// Interactive REPL mode
    Interactive {
        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
    },
}

/// Read prompt from stdin if prompt is "-" or stdin is piped
fn read_prompt(prompt: &str) -> Result<String, anyhow::Error> {
    if prompt == "-" || !io::stdin().is_terminal() {
        // Read from stdin
        let stdin = io::stdin();
        let mut lines = Vec::new();
        for line in stdin.lock().lines() {
            lines.push(line?);
        }
        let input = lines.join("\n");
        if input.trim().is_empty() {
            anyhow::bail!("No input provided via stdin");
        }
        Ok(input)
    } else {
        Ok(prompt.to_string())
    }
}

/// Check if stdout is being piped (not a terminal)
fn is_piped_output() -> bool {
    !io::stdout().is_terminal()
}

pub async fn handle_chat(
    prompt: String,
    model: Option<String>,
    no_thinking: bool,
    raw: bool,
) -> Result<(), anyhow::Error> {
    let prompt = read_prompt(&prompt)?;
    
    // Auto-enable raw mode if output is piped
    let raw = raw || is_piped_output();

    let config = OpenRouterConfig::from_env()?;
    let config = match model {
        Some(m) => config.with_model(m),
        None => config,
    };

    let mut agent = JarvisAgent::new(config)?.with_shell_tool();
    let mut stream = agent.run_stream(&prompt).await?;

    while let Some(event) = stream.next().await {
        match event {
            Ok(StreamEvent::Text(text)) => {
                print!("{}", text);
                io::stdout().flush()?;
            }
            Ok(StreamEvent::Reasoning(thought)) => {
                if !no_thinking && !raw {
                    eprintln!("\n**Thinking**\n{}\n", thought);
                }
            }
            Ok(StreamEvent::ToolCall { name, args }) => {
                if !raw {
                    eprintln!("\n**Tool Call**\n`{}`: {}\n", name, args);
                }
            }
            Ok(StreamEvent::ToolResult { call_id, result }) => {
                if !raw {
                    eprintln!("\n**Tool Result** ({})\n{}\n", call_id, result);
                }
            }
            Ok(StreamEvent::Done) => {
                if !raw {
                    println!();
                }
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

pub async fn handle_run(
    prompt: String,
    model: Option<String>,
    json: bool,
    raw: bool,
) -> Result<(), anyhow::Error> {
    let prompt = read_prompt(&prompt)?;
    
    // Auto-enable raw mode if output is piped (unless JSON is requested)
    let raw = raw || (is_piped_output() && !json);

    let config = OpenRouterConfig::from_env()?;
    let config = match model {
        Some(m) => config.with_model(m),
        None => config,
    };

    let mut agent = JarvisAgent::new(config)?.with_shell_tool();
    let response = agent.run(&prompt).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "response": response
        }))?);
    } else if raw {
        // No trailing newline for raw mode - allows clean piping
        print!("{}", response);
        io::stdout().flush()?;
    } else {
        println!("{}", response);
    }

    Ok(())
}

pub async fn handle_interactive(model: Option<String>) -> Result<(), anyhow::Error> {
    // Interactive mode requires a terminal
    if !io::stdin().is_terminal() {
        anyhow::bail!("Interactive mode requires a terminal. Use 'chat' or 'run' for piped input.");
    }

    println!("**Jarvis Interactive Mode**");
    println!("Type 'exit' to quit, 'clear' to clear history\n");

    let config = OpenRouterConfig::from_env()?;
    let config = match model {
        Some(m) => config.with_model(m),
        None => config,
    };

    let mut agent = JarvisAgent::new(config)?.with_shell_tool();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            println!("Goodbye!");
            break;
        }

        if input == "clear" {
            agent.clear_history();
            println!("History cleared.\n");
            continue;
        }

        println!();
        let mut stream = agent.run_stream(&input).await?;

        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::Text(text)) => {
                    print!("{}", text);
                    io::stdout().flush()?;
                }
                Ok(StreamEvent::Reasoning(thought)) => {
                    println!("\n**Thinking**\n{}\n", thought);
                }
                Ok(StreamEvent::ToolCall { name, args }) => {
                    println!("\n**Tool Call**\n`{}`: {}\n", name, args);
                }
                Ok(StreamEvent::ToolResult { call_id, result }) => {
                    println!("\n**Tool Result** ({})\n{}\n", call_id, result);
                }
                Ok(StreamEvent::Done) => {
                    println!("\n---\n");
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}
