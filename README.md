# AiMitra 🤖

**Your Local AI Code Assistant** — Runs entirely on your machine. No cloud, no API keys, complete privacy.

![AiMitra Screenshot](docs/screenshot.png)

## Features

- 🔒 **100% Local** — Your code never leaves your machine
- 💰 **Zero Cost** — No API keys, no subscriptions, no usage limits
- 🇮🇳 **Built for India** — Optimized for entry-level laptops (8GB RAM, no GPU)
- ⚡ **Fast** — Quantized models via llama.cpp, CPU-optimized
- 🎨 **Custom Branded** — Your own desktop app, not a web wrapper
- 🧩 **Extensive Plugin Support** — Built-in plugins with PonyTail integration
- 🛠️ **Developer Tools** — Slash commands, skills, and IDE integration

## Supported Models (Optimized for 8GB RAM)

| Model | Size | RAM Needed | Best For | Tags |
|-------|------|------------|----------|------|
| **CodeQwen 1.5B** | 1.1 GB | ~3 GB | General coding, fast responses | code, fast, low-ram |
| **DeepSeek Coder 1.3B** | 0.9 GB | ~2.5 GB | Code completion, infilling | code, infilling, low-ram |
| **Phi-3 Mini 4K** | 2.3 GB | ~4 GB | Reasoning, explanations, chat | reasoning, chat, general |
| **Stable Code 3B** | 1.8 GB | ~3.5 GB | Balanced code tasks | code, fill-in-middle |
| **CodeLlama 7B** | 4.0 GB | ~7 GB | General purpose coding | code, general |
| **Mistral 7B** | 4.0 GB | ~7 GB | Reasoning, chat | reasoning, chat |
| **Gemma 2 9B** | 5.5 GB | ~10 GB | Open-ended tasks | general, reasoning |
| **Qwen2.5 Coder** | 7.0 GB | ~12 GB | Advanced code generation | code, advanced |
| **Open Hermes 7B** | 4.0 GB | ~7 GB | Conversational AI | chat, general |
| **WizardCoder** | 3.5 GB | ~6 GB | Code writing | code, writing |
| **StarCoder2 3B** | 2.0 GB | ~4 GB | Code completion | code, completion |
| **Yi Coder 1.5B** | 1.2 GB | ~3 GB | Lightweight coding | code, light |
| **Arcore 7B** | 4.0 GB | ~7 GB | Creative coding | creative, code |
| **Mini Starcoder2** | 1.5 GB | ~3 GB | Fast completions | fast, code |
| **DeepSeek V2 Lite** | 2.0 GB | ~4 GB | Budget-friendly | budget, code |

All models use **Q4_K_M quantization** — best quality/speed tradeoff for CPU inference.

## Quick Start

### Prerequisites

- **Node.js 20+** and **npm/pnpm/yarn**
- **Rust 1.75+** (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Python 3.8+** (for some scripts)
- **Windows 10/11**, macOS 12+, or Linux

### Development

```bash
# Clone and install
cd aimitra
npm install

# Start dev server (frontend + Tauri)
npm run tauri:dev
```

### Build Installer

```bash
# Build production installer (Windows: .exe, macOS: .dmg, Linux: .AppImage/.deb)
npm run tauri:build
```

Output: `src-tauri/target/release/bundle/`

### Download Models Manually

```bash
# Download recommended models to ./models/
npm run download:models
```

Or download from the **Models** tab in the app.

## Architecture

```
┌─────────────────────────────────────────────┐
│           Tauri Desktop App                 │
│  ┌─────────────────┐  ┌─────────────────┐  │
│  │  React Frontend │  │  Rust Backend   │  │
│  │  (TypeScript)   │◄─►│  (Commands)     │  │
│  └────────┬────────┘  └────────┬────────┘  │
│           │                    │           │
│           ▼                    ▼           │
│  ┌─────────────────────────────────────┐  │
│  │         llama.cpp Server            │  │
│  │    (llama-server sidecar binary)    │  │
│  │    - Quantized GGUF models          │  │
│  │    - OpenAI-compatible API          │  │
│  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **Backend**: Rust + Tauri 2 + Axum (local API server)
- **Inference**: llama.cpp (`llama-server`) as embedded sidecar
- **Communication**: Tauri IPC + HTTP/WebSocket for streaming

## Project Structure

```
aimitra/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # UI components
│   │   ├── ChatPanel.tsx   # Main chat interface
│   │   ├── ModelSelector.tsx # Model management
│   │   ├── SettingsPanel.tsx # All settings
│   │   └── StatusBar.tsx   # System stats
│   ├── hooks/              # Custom React hooks
│   ├── lib/                # Utilities, model registry
│   └── App.tsx             # Main app shell
├── src-tauri/              # Rust backend
│   ├── src/main.rs         # Tauri commands, model mgmt, API server
│   ├── src/protocol/       # Protocol conversion (OpenAI/Anthropic/Responses)
│   ├── Cargo.toml          # Rust dependencies
│   └── build.rs            # Build configuration
├── scripts/                # Utility scripts
│   └── download-models.ts  # Model download helper
├── models/                 # GGUF models (gitignored)
├── skills/                 # Skill definitions
├── plugins/                # Plugin definitions
├── eval/                   # Evaluation datasets
├── tauri.conf.json         # Tauri config
└── package.json
```

## Available Plugins

| Plugin | Status | Description |
|--------|--------|-------------|
| shell | ✅ Built-in | Execute shell commands |
| fs | ✅ Built-in | File system operations |
| dialog | ✅ Built-in | File dialogs |
| opener | ✅ Built-in | Open URLs/files with default app |
| process | ✅ Built-in | Process management |
| os | ✅ Built-in | OS information |
| clipboard | ✅ Built-in | Clipboard access |
| notification | ✅ Built-in | Desktop notifications |
| updater | ✅ Built-in | Auto-updates |
| ponytail | ✅ Built-in | Enhanced plugin support |

## Developer Tools

### Slash Commands
- `/review` - Code review suggestions
- `/commit` - Git commit message generation
- `/compact` - Context compression
- `/cost` - Token usage tracking
- `/doctor` - System diagnostics
- `/tasks` - Task management

### Skills System
Skills allow custom workflows. Find them in the `skills/` directory.

### IDE Integration
Configure VSCode extension to connect to AiMitra's local API at `http://localhost:11434`.

## Configuration

Settings are stored in:
- **Windows**: `%APPDATA%\aimitra\settings.json`
- **macOS**: `~/Library/Application Support/aimitra/settings.json`
- **Linux**: `~/.config/aimitra/settings.json`

Key settings:
- Model parameters (temperature, top-p, context length)
- System performance (threads, GPU layers, batch size)
- UI preferences (theme, auto-scroll)
- Local API server (port, enable/disable)

## Local API Server

When enabled (default: port 11434), AiMitra exposes an **OpenAI-compatible API**:

```bash
# Test with curl
curl http://localhost:11434/v1/completions \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Write a Python hello world", "max_tokens": 100}'

# List models
curl http://localhost:11434/v1/models
```

Works with:
- **Continue.dev** (VS Code extension) — Set "Local LLM" provider
- **CodeGPT** — Custom endpoint
- Any OpenAI-compatible client

## Building llama.cpp (for development)

```bash
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
mkdir build && cd build
cmake .. -DLLAMA_BUILD_SERVER=ON
cmake --build . --config Release -j$(nproc)
# Copy llama-server to PATH or ./src-tauri/resources/
```

Pre-built binaries: https://github.com/ggerganov/llama.cpp/releases

## Distribution

### Windows (NSIS Installer)
- Creates `AiMitra Setup 0.1.0.exe`
- Installs to `%LOCALAPPDATA%\Programs\AiMitra`
- Adds Start Menu shortcut
- Auto-detects WebView2, downloads if needed

### macOS (DMG)
- Universal binary (Intel + Apple Silicon)
- Notarization ready (add Apple Developer cert)

### Linux (AppImage + DEB)
- Runs on Ubuntu 20.04+, Fedora 35+, Arch, etc.
- Requires `libwebkit2gtk-4.1`, `libssl3`

## Roadmap

- [x] Protocol Conversion Layer
- [x] MCP Server Integration
- [x] Extended Model Registry (15+ models)
- [x] Slash Commands
- [x] IDE Bridge
- [x] Agent Tools
- [ ] Codebase indexing — Local RAG for your projects
- [ ] Inline completions — VS Code extension with ghost text
- [ ] Indic language support — Hindi/Tamil/Telugu code comments
- [ ] Team sharing — Encrypted model sync via LAN

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines and how to contribute.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

## License

MIT License — Free for personal and commercial use.

## Acknowledgments

- [llama.cpp](https://github.com/ggerganov/llama.cpp) — Incredible CPU inference
- [Tauri](https://tauri.app/) — Tiny, secure desktop apps
- [Qwen](https://github.com/QwenLM/CodeQwen), [DeepSeek](https://github.com/deepseek-ai), [Microsoft Phi-3](https://github.com/microsoft/phi-3) — Open models

---

## Contributors

- **Claude Code** - Architecture patterns, tool system, MCP, skills, plugins
- **Manish Gowda** - Project founder & primary developer

**Made with ❤️ for Indian developers who value privacy and freedom.**

[GitHub Stars](https://github.com/manishn1877/AiMitra/stargazers) | [Issues](https://github.com/manishn1877/AiMitra/issues) | [Download](https://github.com/manishn1877/AiMitra/releases)