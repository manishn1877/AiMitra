# AiMitra - Quick Start Guide

## 🚀 1-Minute Development Start (Recommended for Daily Work)

**Get coding FAST - no complex builds needed!**

```powershell
cd C:\Users\Manis\AiMitra
npm install        # One-time setup (installs Node deps only)
npm run tauri:dev  # Start development mode (hot-reload enabled)
```

**What happens:**
1. Opens Vite dev server + Tauri window immediately
2. Frontend changes update instantly (save → see change)
3. Backend changes trigger fast Rust recompile
4. Connect VS Code to edit files while app runs
5. Total startup time: <30 seconds

---

## 🔌 Connecting VS Code for Development

While `npm run tauri:dev` is running in your terminal:

1. **Open VS Code**
2. **File → Open Folder** → Select your project folder
3. **Extensions to install** (search in VS Code Marketplace):
   - `Tauri` (by Tauri Studios) - essential
   - `ESLint` - for code quality
   - `Tailwind CSS IntelliSense` - for styling
   - `Rust Analyzer` - for backend code

4. **Development workflow:**
   - Edit files in VS Code
   - Save (Ctrl+S)
   - See changes instantly in the running Tauri window
   - No need to restart or rebuild!

---

## 📦 Creating the .exe Installer (For Distribution)

**Only run this when you want to share the app:**

```powershell
npm run tauri:build
```

**Output locations:**
- NSIS Installer: `src-tauri/target/release/bundle/nsis/AiMitra_0.1.0_x64-setup.exe`
- MSI Installer: `src-tauri/target/release/bundle/msi/AiMitra_0.1.0_x64_en-US.msi`

**Features:**
- Creates Start Menu shortcut
- Auto-downloads WebView2 if missing
- Includes uninstaller

---

## ⚡ Development Shortcuts

| Command | What It Does | When to Use |
|---------|--------------|-------------|
| `npm run tauri:dev` | Full dev mode (hot-reload) | Daily coding |
| `npm run dev` | Frontend only (Vite) | Pure UI tweaks |
| `cd src-tauri && cargo check` | Quick Rust type check | Backend logic |
| `npm run tauri:build` | Create installer | Sharing app |

---

## 🛠️ Troubleshooting Made Simple

| Issue | Quick Fix |
|-------|-----------|
| "llama-server not found" | Run `npm run tauri:dev` once - it will guide you |
| Port conflict | Change port in Settings → API Server → Port |
| White screen | Check terminal for errors - fix and save |
| VS Code not detecting Rust | Install "Rust Analyzer" extension |

---

## 💡 Pro Tips

1. **First-time model setup:**
   - Go to Models tab → Download Qwen2.5 Coder 1.5B (1.1GB, fastest)
   - Click "Use" → Go to Chat tab → Start coding!

2. **VS Code debugging:**
   - Press F12 in the Tauri window to open DevTools
   - Set breakpoints in VS Code for frontend code
   - Use `console.log()` in Rust files - see output in terminal

3. **File locations for editing:**
   - **UI/UX:** `src/components/` (React components)
   - **Styles:** `src/index.css` or `tailwind.config.js`
   - **Backend:** `src-tauri/src/` (Rust logic)
   - **Models:** `src/lib/models.ts` (add custom models)

---

## 🎯 Why This Approach?

- **No waiting:** Start coding in <1 minute instead of 15+ minutes for initial build
- **Zero risk:** Development mode won't affect your ability to build .exe later
- **Instant feedback:** See changes immediately - no build cycles
- **VS Code integration:** Edit, debug, and test seamlessly
- **Build only when ready:** Create .exe only when sharing with others

**Happy coding!** Your local AI assistant is ready to help you build amazing things. 🚀