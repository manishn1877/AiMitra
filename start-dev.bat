@echo off
REM AiMitra - One-Click Development Starter
REM Makes it simple to open, start, and connect to VS Code

echo.
echo 🚀 Starting AiMitra Development Mode...
echo.

REM Change to project directory
cd /d "%~dp0"

REM Check if node_modules exists, if not install dependencies
if not exist node_modules (
    echo 📦 Installing Node.js dependencies (first time only)...
    npm install
    if errorlevel 1 (
        echo ❌ npm install failed. Please check Node.js is installed.
        pause
        exit /b 1
    )
)

REM Start development mode
echo ▶️  Launching application with hot-reload...
echo.
echo 💡 Tip: Keep this terminal open while coding
echo 💡 Tip: Open VS Code separately and open this folder
echo.
npm run tauri:dev

REM Keep window open if npm exits unexpectedly
if errorlevel 1 (
    echo.
    echo ⚠️  Development mode ended unexpectedly.
    echo 🔍 Check the output above for errors.
    pause
)