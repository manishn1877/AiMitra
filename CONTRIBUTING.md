# Contributing to AiMitra 🤖

Thank you for your interest in contributing to AiMitra! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Architecture Overview](#architecture-overview)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Building](#building)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. Be constructive, patient, and helpful to others.

## Development Setup

### Prerequisites

1. **Node.js 20+** - Install from [nodejs.org](https://nodejs.org)
2. **Rust 1.75+** - Install with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. **Git** - For version control
4. **Tauri Build Dependencies**:
   - **Windows**: Visual Studio Build Tools, CMake, NWJS
   - **macOS**: Xcode Command Line Tools, CMake
   - **Linux**: `libwebkit2gtk-4.1-dev`, `libssl3`, `cmake`, `build-essential`

### Setup Steps

```bash
# 1. Fork the repository
# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/aimitra.git
cd aimitra

# 3. Install frontend dependencies
npm install

# 4. Build the frontend
npm run build

# 5. Start development mode
npm run tauri:dev
```

### Development Tools

- **Primary**: VS Code with Rust Analyzer and ESLint extensions
- **Hot Reload**: Frontend auto-reloads with Vite
- **Tauri Inspector**: For debugging desktop app

## Project Structure

```
aimitra/
├── src/                    # Frontend (React + TypeScript)
│   ├── App.tsx             # Main application shell
│   ├── components/         # Reusable UI components
│   │   ├── ChatPanel.tsx   # Chat interface
│   │   ├── ModelSelector.tsx # Model management UI
│   │   ├── SettingsPanel.tsx # Settings configuration
│   │   └── StatusBar.tsx   # System status display
│   ├── hooks/              # Custom React hooks
│   │   └── useCompletions.ts # AI completion handling
│   ├── lib/                # Utility libraries
│   │   ├── models.ts       # Model registry
│   │   └── utils.ts        # Helper functions
│   └── index.css           # Tailwind styles
├── src-tauri/              # Rust backend
│   ├── src/main.rs         # Main application logic
│   ├── Cargo.toml          # Rust dependencies
│   └── build.rs            # Build configuration
├── scripts/                # Utility scripts
│   └── download-models.ts  # Model download helper
├── public/                 # Static assets
├── dist/                   # Built frontend (gitignored)
└── models/                 # GGUF models (gitignored)
```

## Architecture Overview

### Frontend Architecture

- **State Management**: React hooks with Zustand for global state
- **UI Framework**: Tailwind CSS with custom components
- **Data Flow**: Unidirectional data flow through props and callbacks
- **API Communication**: Tauri invoke for backend communication

### Backend Architecture

- **Language**: Rust 2021 edition
- **Framework**: Tauri for desktop bridge, Axum for HTTP API
- **Concurrency**: Tokio for async runtime, parking_lot for mutexes
- **Process Management**: Llama.cpp server as sidecar process
- **Communication**: HTTP for API, IPC for Tauri commands

### Data Flow

```
User Input → ChatPanel → useCompletions Hook → Tauri Invoke → Rust Backend
       ↓                                                ↓
   Response ← Streaming Events ← HTTP API ← Llama.cpp Server
```

## How to Contribute

### Reporting Bugs

1. Search existing issues to avoid duplicates
2. Create a new issue with:
   - Clear description of the bug
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Screenshots if applicable
   - System information (OS, RAM, CPU)

### Suggesting Features

1. Open an issue with `enhancement` label
2. Describe the feature clearly
3. Explain the use case and benefits
4. Consider providing a mockup or example

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Add tests if applicable
5. Commit with clear messages: `git commit -m "Add my feature"`
6. Push to branch: `git push origin feature/my-feature`
7. Open a Pull Request

### Coding Style

See [Coding Standards](#coding-standards) section below.

## Coding Standards

### Frontend (TypeScript/React)

- Use TypeScript for type safety
- Follow existing code style in the project
- Use functional components with hooks
- Write self-documenting code with clear names
- Add JSDoc comments for public functions
- Use ESLint for code quality

### Backend (Rust)

- Follow Rust API Guidelines
- Use meaningful variable names
- Handle errors with `Result` and `?` operator
- Add doc comments for public functions
- Use `rustfmt` for consistent formatting
- Run `cargo clippy` for linting

### Git Commit Messages

Follow conventional commits:

```
feat: add new model support
fix: resolve model loading crash
docs: update contributing guidelines
refactor: restructure protocol module
test: add unit tests for converter
```

## Pull Request Process

1. **Update Documentation** if you change functionality
2. **Add Tests** for new features or bug fixes
3. **Ensure CI Passes** - All checks must pass
4. **Request Review** from maintainers
5. **Address Feedback** from reviewers

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Commit messages are clear and conventional
- [ ] All CI checks pass

## Testing

### Frontend Testing

```bash
# Run tests
npm test

# Run TypeScript check
npx tsc --noEmit
```

### Backend Testing

```bash
# Run tests
cargo test

# Run clippy linter
cargo clippy -- -D warnings
```

### Manual Testing

For model loading and inference:
1. Build the frontend: `npm run build`
2. Start Tauri dev: `npm run tauri:dev`
3. Test model download and usage
4. Verify API server functionality at `http://localhost:11434`

## Building

### Development Build

```bash
npm run tauri:dev
```

### Production Build

```bash
# Build frontend
npm run build

# Build Tauri app
npm run tauri:build
```

### Build Artifacts

After building, you'll find installers in:
- `src-tauri/target/release/bundle/`

### Build Targets

To build for specific platforms (cross-compilation may require additional setup):

```bash
# Windows (on Windows)
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS (on macOS)
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin

# Linux (on Linux)
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## Reporting Issues

When reporting issues, please include:

1. **AiMitra Version** - Check in About dialog or `tauri.conf.json`
2. **Operating System** - Windows 10/11, macOS 12+, Linux distro
3. **Hardware** - CPU, RAM, GPU (if applicable)
4. **Steps to Reproduce** - Exact steps taken
5. **Expected Result** - What you expected to happen
6. **Actual Result** - What actually happened
7. **Logs** - Console output from developer tools

### Getting Logs

1. Press `Ctrl+Shift+I` (or `Cmd+Option+I` on macOS) to open DevTools
2. Go to Console tab
3. Reproduce the issue
4. Copy and paste relevant log messages

### Contact

For questions or discussions, open a GitHub Discussion or reach out via:
- GitHub Issues: https://github.com/manishn1877/AiMitra/issues
- GitHub Discussions: https://github.com/manishn1877/AiMitra/discussions

### Acknowledgments

This project incorporates patterns and techniques from:
- **Claude Code** - Architecture patterns, tool system, MCP, skills, plugins
- **DeepSeek Recipe** - Protocol conversion, prompt rendering, structured output
- **DeepSeek Harness** - Evaluation framework, benchmarking tools
- **DeepInfer** - High-performance inference optimization patterns

---

Thank you for contributing to AiMitra! 🙌