use jarvis_core::{JarvisAgent, OpenRouterConfig, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), jarvis_core::Error> {
    // Load configuration from environment
    let config = OpenRouterConfig::from_env()?;

    // Create agent with shell tool enabled
    let mut agent = JarvisAgent::new(config)?
        .with_shell_tool()
        .with_preamble("You are a helpful assistant. When asked to perform system tasks, use the shell tool.");

    println!("**Jarvis Basic Example**\n");
    println!("Model: {}\n", agent.config().model);

    let prompt = "What is 2 + 2?";
    println!("User: {}\n", prompt);
    println!("Assistant:");

    // Run streaming
    let mut stream = agent.run_stream(prompt).await?;

    while let Some(event) = stream.next().await {
        match event {
            Ok(StreamEvent::Text(text)) => print!("{}", text),
            Ok(StreamEvent::Reasoning(thought)) => println!("\n**Thinking**\n{}\n", thought),
            Ok(StreamEvent::ToolCall { name, args }) => {
                println!("\n**Tool Call**\n{}: {}\n", name, args);
            }
            Ok(StreamEvent::ToolResult { call_id, result }) => {
                println!("\n**Tool Result** ({})\n{}\n", call_id, result);
            }
            Ok(StreamEvent::Done) => {
                println!("\n\n---\n");
                break;
            }
            Err(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
        }
    }

    Ok(())
}
