@echo off
REM AiMitra - Quick Setup Batch File
REM Run this as Administrator for best results

echo ╔══════════════════════════════════════════════════════════════╗
echo ║          AiMitra - Windows Setup                         ║
echo ║          Local AI Code Assistant for 8GB RAM Laptops        ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

REM Check if running as admin
net session >nul 2>&1
if %errorLevel% == 0 (
    echo [INFO] Running as Administrator ✓
) else (
    echo [WARN] Not running as Administrator - some installs may fail
    echo [INFO] Right-click Command Prompt and "Run as Administrator"
    echo.
)

REM Run the PowerShell setup script
powershell -ExecutionPolicy Bypass -File "%~dp0setup-windows.ps1"

echo.
echo Setup complete! Press any key to exit...
pause >nul