# Jarvis AI Agent Project Plan

## Goal

Build a Rust-based AI agent library using rig-core that supports:
- CLI interface for local use
- HTTP/WebSocket server for remote clients (any language/platform)
- Streaming responses with tool execution
- Shell command tool with safety features

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      jarvis (Workspace)                  │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐            │
│  │  jarvis-core     │  │   jarvis-cli     │            │
│  │  (Library)       │  │   (Binary)       │            │
│  └──────────────────┘  └──────────────────┘            │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │              jarvis-server (Binary)               │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## Configuration Decisions

| Setting | Decision | Rationale |
|---------|----------|-----------|
| Library name | `jarvis-core` | Clear, descriptive |
| Default model | `google/gemini-2.5-flash` | Fast, good reasoning, cost-effective |
| Shell tool | Opt-in (`.with_shell_tool()`) | Security - users choose when to enable |
| Async runtime | Runtime-agnostic | Flexibility for consumers |
| Error handling | Custom `jarvis_core::Error` with thiserror | Type-safe, user-friendly errors |
| Thinking output | Show with **Thinking** header | Transparency in model reasoning |

---

## Project Structure

```
jarvis/
├── Cargo.toml                    # Workspace (resolver = "2")
├── README.md                     # User documentation
├── BUILDING.md                   # Build instructions
├── AgentProjectPlan.md           # This file
├── .env.example                  # Environment template
├── .gitignore
│
├── jarvis-core/                  # Library crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs               # Exports all modules
│   │   ├── error.rs             # Custom error types
│   │   ├── agent/
│   │   │   ├── mod.rs           # JarvisAgent, AgentStream
│   │   │   ├── jarvis_agent.rs  # Core streaming agent
│   │   │   └── message.rs       # Message types
│   │   ├── tools/
│   │   │   ├── mod.rs
│   │   │   └── shell.rs         # ShellExecute tool
│   │   ├── streaming/
│   │   │   └── mod.rs           # JarvisStream, StreamEvent
│   │   └── providers/
│   │       ├── mod.rs
│   │       └── openrouter.rs    # OpenRouter config
│   └── examples/
│       └── basic_usage.rs
│
├── jarvis-cli/                   # CLI binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── cli/
│           ├── mod.rs
│           └── commands.rs      # Chat, Run, Interactive
│
└── jarvis-server/                # HTTP/WebSocket server
    ├── Cargo.toml
    └── src/
        ├── lib.rs               # Exports
        ├── main.rs              # Server CLI
        ├── server.rs            # ServerConfig, run_server()
        ├── handlers/
        │   ├── mod.rs           # ChatRequest, ServerState
        │   └── http_handlers.rs # HTTP endpoints
        └── websocket/
            └── mod.rs           # WebSocket handler
```

---

## Core API (jarvis-core)

### JarvisAgent

```rust
pub struct JarvisAgent { /* ... */ }

impl JarvisAgent {
    pub fn new(config: OpenRouterConfig) -> Result<Self>;
    pub fn with_shell_tool(self) -> Self;
    pub fn with_model(self, model: impl Into<String>) -> Self;
    pub fn with_temperature(self, temperature: f32) -> Self;
    pub fn with_preamble(self, preamble: impl Into<String>) -> Self;
    pub async fn run(&self, prompt: &str) -> Result<String>;
    pub fn run_stream<'a>(&'a self, prompt: &'a str) -> JarvisStream<'a>;
}
```

### StreamEvent

```rust
pub enum StreamEvent {
    Text(String),
    Reasoning(String),
    ToolCall { name: String, args: String },
    ToolResult { call_id: String, result: String },
    Done,
}
```

### OpenRouterConfig

```rust
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model: String,
    pub site_url: Option<String>,
    pub app_name: Option<String>,
    pub temperature: Option<f32>,
    pub preamble: Option<String>,
    pub base_url: String,
}

impl OpenRouterConfig {
    pub fn from_env() -> Result<Self>;
    pub fn builder() -> OpenRouterConfigBuilder;
    pub fn with_model(self, model: impl Into<String>) -> Self;
    pub fn with_temperature(self, temperature: f32) -> Self;
}
```

---

## ShellExecute Tool

```rust
pub struct ShellExecute;

impl ShellExecute {
    pub const NAME: &'static str = "execute_shell_command";
    pub async fn call(&self, args: ShellArgs) -> Result<ShellResult>;
}

#[derive(JsonSchema)]
pub struct ShellArgs {
    pub command: String,
    pub cwd: Option<String>,
    pub timeout: Option<u64>,
}

pub struct ShellResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub success: bool,
}
```

**Safety Features:**
- 30-second timeout (default, configurable)
- `kill_on_drop` prevents orphaned processes
- Working directory restriction support

---

## CLI Commands (jarvis-cli)

```bash
# Chat with streaming
jarvis-cli chat "Your prompt" --model google/gemini-2.5-flash --no-thinking

# Non-streaming response
jarvis-cli run "Your prompt" --no-stream --json

# Interactive REPL mode
jarvis-cli interactive

# Options
--model, -m    # Model to use
--no-stream    # Disable streaming
--no-thinking  # Hide thinking output
--json         # Output as JSON
```

### Interactive Mode Commands

- `exit` - Quit interactive mode
- `clear` - Clear conversation history
- `model <name>` - Change the model

---

## Server API (jarvis-server)

### Endpoints

| Method | Endpoint | Description | Response |
|--------|----------|-------------|----------|
| GET | `/health` | Health check | `OK` |
| POST | `/api/chat` | Streaming chat (SSE) | Server-Sent Events |
| POST | `/api/chat/json` | Non-streaming JSON | `{"response": "...", "done": true}` |
| GET | `/api/models` | List models | `{"models": [...], "default": "..."}` |
| WS | `/ws` | WebSocket streaming | JSON events |

### SSE Events

```
event: text
data: The response text...

event: thinking
data: Model reasoning...

event: tool_call
data: {"name":"...", "args":"..."}

event: done
data: {"done":true}
```

### WebSocket Protocol

```javascript
// Send
ws.send(JSON.stringify({ prompt: "Your question", model: "..." }));

// Receive
{ "type": "text", "content": "..." }
{ "type": "thinking", "content": "..." }
{ "type": "tool_call", "name": "...", "args": "..." }
{ "type": "done" }
```

---

## Client Examples

### JavaScript (WebSocket)

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    ws.send(JSON.stringify({ prompt: 'List files' }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    switch (data.type) {
        case 'text': process.stdout.write(data.content); break;
        case 'thinking': console.log(`\n**Thinking**\n${data.content}\n`); break;
        case 'tool_call': console.log(`\n**Tool:** ${data.name}\n`); break;
        case 'done': console.log('\n---\n'); break;
    }
};
```

### Python (WebSocket)

```python
import asyncio, websockets, json

async def chat():
    async with websockets.connect('ws://localhost:8080/ws') as ws:
        await ws.send(json.dumps({'prompt': 'List files'}))
        async for msg in ws:
            data = json.loads(msg)
            if data['type'] == 'text': print(data['content'], end='', flush=True)
            elif data['type'] == 'thinking': print(f"\n**Thinking**\n{data['content']}\n")
            elif data['type'] == 'tool_call': print(f"\n**Tool:** {data['name']}\n")
            elif data['type'] == 'done': print('\n---\n')

asyncio.run(chat())
```

### cURL (HTTP SSE)

```bash
curl -X POST http://localhost:8080/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "List files", "stream": true}'
```

### cURL (JSON)

```bash
curl -X POST http://localhost:8080/api/chat/json \
  -H "Content-Type: application/json" \
  -d '{"prompt": "What is Rust?"}'
```

---

## Dependencies

### jarvis-core

| Crate | Version | Purpose |
|-------|---------|---------|
| rig-core | 0.27.0 | LLM framework |
| rig-derive | 0.1.10 | Tool macros |
| tokio | 1.x | Async runtime |
| futures | 0.3 | Stream utilities |
| async-stream | 0.3 | Async stream macros |
| thiserror | 2.0 | Error types |
| chrono | 0.4 | Timestamps |
| schemars | 0.8 | JSON schemas |
| url | 2.5 | URL handling |
| async-trait | 0.1 | Runtime-agnostic |

### jarvis-cli

| Crate | Version | Purpose |
|-------|---------|---------|
| jarvis-core | 0.1.0 | Core library |
| clap | 4.5 | CLI parsing |
| tracing | 0.1 | Logging |

### jarvis-server

| Crate | Version | Purpose |
|-------|---------|---------|
| jarvis-core | 0.1.0 | Core library |
| axum | 0.8 | HTTP server |
| tokio-tungstenite | 0.24 | WebSocket |
| hyper | 1 | HTTP client |
| tower-http | 0.6 | HTTP middleware |

---

## Environment Variables

```env
# Required
OPENROUTER_API_KEY=sk-or-v1-your-api-key

# Optional
OPENROUTER_SITE_URL=https://github.com/yourusername/jarvis
OPENROUTER_APP_NAME=jarvis
```

---

## Build Requirements

- **Rust** 1.70+
- **C compiler** (gcc/cc/clang)
- **OpenRouter** API key

### Why C Compiler?

Many Rust crates wrap C/C++ libraries:
- **OpenSSL** - HTTPS/TLS (rig-core, reqwest)
- **ICU** - Unicode normalization (chrono, url)
- **libc** - System calls (tokio)

---

## Usage Summary

```bash
# Build all
cargo build

# Build specific crates
cargo build -p jarvis-core
cargo build -p jarvis-cli
cargo build -p jarvis-server

# Run CLI
cargo run -p jarvis-cli -- chat "Your prompt"

# Run server
cargo run -p jarvis-server -- start --port 8080 --with-shell

# Run example
cargo run -p jarvis-core --example basic_usage
```

---

## Implementation Status

| Component | Status |
|-----------|--------|
| jarvis-core | ✅ Complete |
| jarvis-cli | ✅ Complete |
| jarvis-server | ✅ Complete |
| Documentation | ✅ Complete |
| Build verification | ⏳ Needs C compiler |

---

**Ready to build on a system with gcc/cc installed.**
