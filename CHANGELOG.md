# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-01-15

### Added

- **AiMitra branding** - Renamed from CodeMitra toAiMitra
- **Extended Model Registry** - Added 11 new models for a total of 15+ models
  - CodeQwen 1.5B (recommended)
  - DeepSeek Coder 1.3B (recommended)
  - Phi-3 Mini 4K (recommended)
  - Stable Code 3B
  - CodeLlama 7B Instruct
  - Mistral 7B Instruct v0.2
  - Gemma 2 9B IT
  - Qwen2.5 Coder 7B
  - Open Hermes 2.5 Mistral 7B
  - WizardCoder 1.5B/7B
  - StarCoder2 3B
  - Yi Coder 1.5B
  - Arcore 7B Inst
  - Mini-Starcoder2 1.5B
  - DeepSeek V2 Lite

- **Protocol Conversion Layer** (from deepseek-recipe)
  - OpenAI Chat Completions API support
  - Anthropic Messages API support
  - OpenAI Responses API support
  - Structured output support (JSON schema)
  - Streaming response handlers

- **Enhanced API Server** (port 11434 by default)
  - OpenAI-compatible completions endpoint
  - Models listing endpoint
  - CORS support for local development
  - Health check endpoint

- **Model Management**
  - Download models with mirror support
  - Progress tracking during downloads
  - SHA256 verification
  - Auto-delete loaded models before deletion

- **System Integration**
  - Real-time system stats monitoring
  - RAM usage tracking
  - CPU usage tracking
  - Token generation speed (tok/s)
  - Thermal state detection (Windows/Linux)

- **PonyTail Plugin Support**
  - Enhanced plugin loading
  - Better plugin lifecycle management

- **Developer Tools**
  - Slash commands: /review, /commit, /compact, /cost, /doctor, /tasks
  - Skills system infrastructure
  - IDE bridge foundation
  - Agent tools (FileRead, Write, Edit, Grep, Glob, Bash, WebSearch, WebFetch, Agent)

- **Cross-platform Support**
  - Windows 10/11
  - macOS 12+
  - Linux (Ubuntu, Fedora, Arch)

- **Plugin System**
  - Created `plugins/` directory with README.md
  - PonyTail plugin integration for enhanced extensibility

- **Skills System**
  - Created `skills/` directory with README.md
  - Infrastructure for custom skill definitions

- **Evaluation Framework**
  - Created `eval/` directory with README.md
  - Infrastructure for benchmarking and testing

### Changed

- **Rebranding**: Changed name from CodeMitra toAiMitra
- Updated all file references
- Updated config directory path
- Updated documentation

### Fixed
- Fixed Rust compilation errors with `parking_lot` mutex handling in async contexts
- Fixed `Send` trait violations by properly managing lock lifetimes
- Fixed unused import/variable warnings
- Fixed channel naming inconsistencies

---

## [0.0.1] - Initial Development

### Added

- Initial project setup with Tauri 2
- React 18 frontend with TypeScript
- Tailwind CSS styling
- Basic chat interface
- Model selection UI
- Settings panel
- Status bar with system metrics
- llama.cpp server integration
- Basic model download support
- 4 recommended models

### Security

- Initial release - No known vulnerabilities