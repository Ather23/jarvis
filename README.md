# Jarvis AI Agent

A Rust-based AI agent library built with rig-core, featuring streaming responses, multi-turn conversations, shell command execution, and HTTP/WebSocket server for remote access via OpenRouter with Gemini models.

> ⚡ *Human-seeded, AI-cultivated. The foundation was laid by flesh and blood; the architecture grew from artificial minds.*

## Features

- **Streaming Responses**: Real-time token-by-token output
- **Multi-turn Conversations**: Maintain conversation history
- **Shell Command Tool**: Execute shell commands with safety features
- **Thinking Display**: Show Gemini's reasoning process
- **HTTP Server**: REST API for any client
- **WebSocket**: Real-time streaming for web apps
- **Runtime Agnostic**: Works with any async runtime (Tokio by default)
- **Modular Design**: Library-first architecture for CLI, servers, or applications

## Architecture

```
jarvis/
├── jarvis-core/           # Library crate
│   ├── src/
│   │   ├── lib.rs         # Exports
│   │   ├── agent/         # JarvisAgent, streaming
│   │   ├── tools/         # ShellExecute tool
│   │   ├── streaming/     # JarvisStream, StreamEvent
│   │   ├── providers/     # OpenRouter config
│   │   └── error.rs       # Custom error types
│   └── examples/
│
├── jarvis-cli/            # CLI binary
│   ├── src/
│   │   ├── main.rs
│   │   └── cli/
│   └── Cargo.toml
│
└── jarvis-server/         # HTTP/WebSocket server
    ├── src/
    │   ├── main.rs
    │   ├── server.rs      # Server configuration
    │   ├── handlers/      # HTTP handlers
    │   └── websocket/     # WebSocket handler
    └── Cargo.toml
```

## Installation

### Prerequisites

- Rust 1.70+
- C compiler (gcc/cc) for building dependencies
- OpenRouter API key

### Setup

```bash
git clone https://github.com/yourusername/jarvis
cd jarvis
cp .env.example .env
nano .env
```

## Usage

### As a Library

```rust
use jarvis_core::{JarvisAgent, OpenRouterConfig, StreamEvent};

#[tokio::main]
async fn main() -> Result<(), jarvis_core::Error> {
    let config = OpenRouterConfig::from_env()?;
    let agent = JarvisAgent::new(config)?.with_shell_tool();

    let stream = agent.run_stream("List files");

    for await event in stream {
        match event {
            Ok(StreamEvent::Text(text)) => print!("{}", text),
            Ok(StreamEvent::Thinking(thought)) => println!("\n**Thinking**\n{}\n", thought),
            Ok(StreamEvent::ToolCall { name, args }) => println!("\n**Tool** {}: {}\n", name, args),
            Ok(StreamEvent::Done) => println!("\n---\n"),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}
```

### CLI Commands

```bash
jarvis-cli chat "Your prompt" --stream
jarvis-cli run "Your prompt" --no-stream
jarvis-cli interactive
```

### Server (HTTP/WebSocket)

```bash
# Start server with shell tool enabled
jarvis-server start --host 0.0.0.0 --port 8080 --with-shell

# Start server without shell tool
jarvis-server start --port 8080
```

## Server API

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/api/chat` | Streaming chat (SSE) |
| POST | `/api/chat/json` | Non-streaming JSON |
| GET | `/api/models` | List models |
| WS | `/ws` | WebSocket streaming |

### SSE Chat Endpoint

```bash
curl -X POST http://localhost:8080/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "List files", "stream": true}'
```

Response (Server-Sent Events):
```
event: text
data: The user wants

event: thinking
data: I need to execute a shell command to list files.

event: tool_call
data: {"name":"execute_shell_command","args":"{\"command\":\"ls -la\"}"}

event: text
data:  total 32
drwxr-xr-x  5 user  user  4096 Dec 29 10:00 .

event: done
data: {"done":true}
```

### JSON Chat Endpoint

```bash
curl -X POST http://localhost:8080/api/chat/json \
  -H "Content-Type: application/json" \
  -d '{"prompt": "What is Rust?"}'
```

Response:
```json
{
  "response": "Rust is a systems programming language...",
  "done": true
}
```

### WebSocket Client Example

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({ prompt: 'List files' }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  switch (data.type) {
    case 'text':
      process.stdout.write(data.content);
      break;
    case 'thinking':
      console.log('\n**Thinking**\n' + data.content + '\n');
      break;
    case 'tool_call':
      console.log('\n**Tool Call:** ' + data.name + '\n');
      break;
    case 'done':
      console.log('\n---\n');
      break;
  }
};
```

### Python WebSocket Client

```python
import asyncio
import websockets
import json

async def chat():
    async with websockets.connect('ws://localhost:8080/ws') as ws:
        await ws.send(json.dumps({'prompt': 'List files'}))
        
        async for message in ws:
            data = json.loads(message)
            if data['type'] == 'text':
                print(data['content'], end='', flush=True)
            elif data['type'] == 'thinking':
                print(f"\n**Thinking**\n{data['content']}\n")
            elif data['type'] == 'tool_call':
                print(f"\n**Tool:** {data['name']}\n")
            elif data['type'] == 'done':
                print('\n---\n')

asyncio.run(chat())
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENROUTER_API_KEY` | OpenRouter API key | Required |
| `OPENROUTER_SITE_URL` | Site URL for HTTP-Referer | Optional |
| `OPENROUTER_APP_NAME` | Application name | `jarvis-core` |

### Default Model

`google/gemini-2.5-flash` (fast, good reasoning). Change via:
- CLI: `--model google/gemini-2.5-pro`
- Code: `.with_model("anthropic/claude-3-5-sonnet")`

## Shell Tool Safety

```rust
let agent = JarvisAgent::new(config)?.with_shell_tool();
```

**Safety Features:**
- 30-second timeout
- `kill_on_drop` prevents orphaned processes
- Working directory restriction support
- All commands logged

## Building

```bash
# Build all crates
cargo build

# Build specific crates
cargo build -p jarvis-core
cargo build -p jarvis-cli
cargo build -p jarvis-server

# Run server
cargo run -p jarvis-server -- start --port 8080 --with-shell

# Run example
cargo run -p jarvis-core --example basic_usage
```

## Dependencies

### jarvis-core
- `rig-core`, `rig-derive`
- `tokio`, `futures`, `async-stream`
- `thiserror`, `chrono`, `schemars`

### jarvis-cli
- `jarvis-core`, `clap`, `tracing`

### jarvis-server
- `jarvis-core`
- `axum`, `tokio-tungstenite`, `hyper`

## License

MIT
