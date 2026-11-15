<#
.SYNOPSIS
    Quick verification script - run after setup to confirm everything works
#>

$ErrorActionPreference = "Stop"

$Green = [ConsoleColor]::Green
$Red = [ConsoleColor]::Red
$Yellow = [ConsoleColor]::Yellow
$Cyan = [ConsoleColor]::Cyan

function Write-OK { param($msg) Write-Host "✅ $msg" -ForegroundColor $Green }
function Write-FAIL { param($msg) Write-Host "❌ $msg" -ForegroundColor $Red }
function Write-WARN { param($msg) Write-Host "⚠️  $msg" -ForegroundColor $Yellow }
function Write-INFO { param($msg) Write-Host "ℹ️  $msg" -ForegroundColor $Cyan }

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor $Cyan
Write-Host "║          Code Mitra - Setup Verification                    ║" -ForegroundColor $Cyan
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor $Cyan
Write-Host ""

$allOk = $true

# 1. Check project structure
Write-INFO "Checking project structure..."
$requiredFiles = @(
    "package.json",
    "src-tauri\Cargo.toml",
    "src-tauri\tauri.conf.json",
    "src-tauri\src\main.rs",
    "src\main.tsx",
    "src\App.tsx",
)

foreach ($f in $requiredFiles) {
    if (Test-Path $f) { Write-OK "Found: $f" }
    else { Write-FAIL "Missing: $f"; $allOk = $false }
}

# 2. Check tools
Write-INFO ""
Write-INFO "Checking toolchain..."
$tools = @(
    @{ Name = "Rust"; Cmd = "rustc --version" },
    @{ Name = "Cargo"; Cmd = "cargo --version" },
    @{ Name = "Node"; Cmd = "node --version" },
    @{ Name = "NPM"; Cmd = "npm --version" },
    @{ Name = "Tauri CLI"; Cmd = "cargo-tauri --version" },
)

foreach ($t in $tools) {
    try {
        $out = & $t.Cmd 2>$null
        Write-OK "$($t.Name): $out"
    } catch {
        Write-FAIL "$($t.Name): Not found"
        $allOk = $false
    }
}

# 3. Check llama-server
Write-INFO ""
Write-INFO "Checking llama-server..."
$llamaPath = "src-tauri\resources\llama-server.exe"
if (Test-Path $llamaPath) {
    Write-OK "llama-server.exe found at $llamaPath"
} else {
    Write-WARN "llama-server.exe NOT found at $llamaPath"
    Write-WARN "  Run setup with -ForceRebuild or build manually from llama.cpp"
    $allOk = $false
}

# 4. Check icons
Write-INFO ""
Write-INFO "Checking icons..."
$icons = @("src-tauri\icons\icon.ico", "src-tauri\icons\icon.png", "src-tauri\icons\32x32.png")
foreach ($icon in $icons) {
    if (Test-Path $icon) { Write-OK "Found: $icon" }
    else { Write-WARN "Missing: $icon (using placeholder)" }
}

# 5. Check node_modules
Write-INFO ""
Write-INFO "Checking npm dependencies..."
if (Test-Path "node_modules") {
    Write-OK "node_modules exists"
} else {
    Write-WARN "node_modules not found - run 'npm install'"
    $allOk = $false
}

# 6. Try TypeScript compile
Write-INFO ""
Write-INFO "Checking TypeScript compilation..."
try {
    npx tsc --noEmit 2>&1 | Out-Null
    Write-OK "TypeScript compiles without errors"
} catch {
    Write-FAIL "TypeScript compilation failed"
    $allOk = $false
}

# 7. Check Cargo.toml syntax
Write-INFO ""
Write-INFO "Checking Cargo.toml..."
try {
    cd src-tauri
    cargo check 2>&1 | Out-Null
    cd ..
    Write-OK "Cargo.toml valid, dependencies resolvable"
} catch {
    Write-FAIL "Cargo check failed"
    $allOk = $false
}

Write-Host ""
if ($allOk) {
    Write-OK "All checks passed! 🎉"
    Write-Host ""
    Write-INFO "Ready to run:"
    Write-INFO "  npm run tauri:dev     # Development mode"
    Write-INFO "  npm run tauri:build   # Build Windows installer"
} else {
    Write-FAIL "Some checks failed. Run setup-windows.ps1 or fix manually."
    exit 1
}