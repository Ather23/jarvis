# Building Jarvis

## Requirements

- **Rust 1.70+**
- **C compiler** (gcc, cc, or clang)
- **OpenRouter API key**

## Install Build Tools

### Ubuntu/Debian
```bash
sudo apt-get update && sudo apt-get install -y build-essential gcc g++
```

### macOS
```bash
xcode-select --install
```

### Windows
```bash
# Install via Visual Studio or MSVC
# Or use: winget install -e --id GNU.GCC
```

## Build Commands

```bash
# Build all crates
cargo build

# Build specific crates
cargo build -p jarvis-core
cargo build -p jarvis-cli
cargo build -p jarvis-server

# Release build
cargo build --release

# Run tests
cargo test

# Run examples
cargo run -p jarvis-core --example basic_usage
```

## Troubleshooting

### "linker `cc` not found"
Install a C compiler (see above).

### "OPENROUTER_API_KEY not set"
Copy `.env.example` to `.env` and add your API key.

### Compilation errors
Ensure you're using a recent Rust version:
```bash
rustup update stable
cargo clean
cargo build
```

## Notes

The project uses native dependencies (OpenSSL, ICU) which require a C compiler. This is a standard requirement for Rust projects with such dependencies.
