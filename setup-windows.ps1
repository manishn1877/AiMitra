<#
.SYNOPSIS
    AiMitra - Complete Windows Setup Script
    Installs all dependencies, builds llama.cpp, and prepares the project for development

.DESCRIPTION
    This script will:
    1. Check/install Rust toolchain
    2. Check/install Node.js (via winget/choco/scoop)
    3. Install Tauri CLI
    4. Build llama.cpp with server support
    5. Install npm dependencies
    6. Create placeholder icons
    7. Verify the build

.NOTES
    Run as Administrator for best results (needed for winget/choco)
    Run in PowerShell 5.1+ or PowerShell 7+
#>

param(
    [switch]$SkipRust,
    [switch]$SkipNode,
    [switch]$SkipLlamaCpp,
    [switch]$SkipNpmInstall,
    [switch]$ForceRebuild
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Colors for output
$Green = [ConsoleColor]::Green
$Yellow = [ConsoleColor]::Yellow
$Red = [ConsoleColor]::Red
$Cyan = [ConsoleColor]::Cyan
$Gray = [ConsoleColor]::DarkGray

function Write-Log {
    param([string]$Message, [ConsoleColor]$Color = $Gray)
    $time = Get-Date -Format "HH:mm:ss"
    Write-Host "[$time] $Message" -ForegroundColor $Color
}

function Write-Success { param([string]$Message) Write-Log "✅ $Message" $Green }
function Write-Warning { param([string]$Message) Write-Log "⚠️  $Message" $Yellow }
function Write-ErrorMsg { param([string]$Message) Write-Log "❌ $Message" $Red }
function Write-Info { param([string]$Message) Write-Log "ℹ️  $Message" $Cyan }

function Test-Command {
    param([string]$Command)
    try {
        $null = & $Command --version 2>$null
        return $true
    } catch {
        return $false
    }
}

function Install-Rust {
    Write-Info "Checking Rust toolchain..."
    if (Test-Command "rustc") {
        $version = rustc --version
        Write-Success "Rust already installed: $version"
        return
    }

    Write-Warning "Rust not found. Installing via rustup..."
    try {
        # Download and run rustup-init
        $rustupUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
        $installer = "$env:TEMP\rustup-init.exe"

        Write-Info "Downloading rustup-init.exe..."
        Invoke-WebRequest -Uri $rustupUrl -OutFile $installer -UseBasicParsing

        Write-Info "Running rustup installer (this may take a few minutes)..."
        & $installer -y --default-toolchain stable --profile minimal --no-modify-path

        # Refresh PATH
        $env:PATH = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

        if (Test-Command "rustc") {
            Write-Success "Rust installed successfully: $(rustc --version)"
        } else {
            Write-ErrorMsg "Rust installation may have succeeded but not in PATH. Please restart terminal."
            Write-Info "Run: \$env:PATH = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')"
        }
    } catch {
        Write-ErrorMsg "Failed to install Rust: $_"
        Write-Info "Manual install: https://rustup.rs/"
        exit 1
    }
}

function Install-Node {
    Write-Info "Checking Node.js..."
    if (Test-Command "node") {
        $version = node --version
        Write-Success "Node.js already installed: $version"
        return
    }

    Write-Warning "Node.js not found. Attempting to install..."

    # Try winget first (Windows 10/11 built-in)
    if (Test-Command "winget") {
        Write-Info "Installing Node.js via winget..."
        try {
            winget install --id OpenJS.NodeJS.LTS --source winget --accept-source-agreements --accept-package-agreements
            if (Test-Command "node") {
                Write-Success "Node.js installed via winget: $(node --version)"
                return
            }
        } catch {
            Write-Warning "winget install failed, trying alternatives..."
        }
    }

    # Try Chocolatey
    if (Test-Command "choco") {
        Write-Info "Installing Node.js via Chocolatey..."
        try {
            choco install nodejs-lts -y
            if (Test-Command "node") {
                Write-Success "Node.js installed via Chocolatey: $(node --version)"
                return
            }
        } catch {
            Write-Warning "Chocolatey install failed..."
        }
    }

    # Try Scoop
    if (Test-Command "scoop") {
        Write-Info "Installing Node.js via Scoop..."
        try {
            scoop install nodejs-lts
            if (Test-Command "node") {
                Write-Success "Node.js installed via Scoop: $(node --version)"
                return
            }
        } catch {
            Write-Warning "Scoop install failed..."
        }
    }

    Write-ErrorMsg "Could not install Node.js automatically."
    Write-Info "Please install manually from: https://nodejs.org/ (LTS version)"
    exit 1
}

function Install-TauriCLI {
    Write-Info "Checking Tauri CLI..."
    if (Test-Command "cargo-tauri") {
        Write-Success "Tauri CLI already installed"
        return
    }

    Write-Info "Installing Tauri CLI v2..."
    try {
        cargo install tauri-cli --version "^2" --locked
        Write-Success "Tauri CLI installed"
    } catch {
        Write-ErrorMsg "Failed to install Tauri CLI: $_"
        exit 1
    }
}

function Build-LlamaCpp {
    Write-Info "Checking llama.cpp..."

    $llamaServerPath = "$PSScriptRoot\src-tauri\resources\llama-server.exe"
    if ((Test-Path $llamaServerPath) -and (-not $ForceRebuild)) {
        Write-Success "llama-server.exe already exists at $llamaServerPath"
        return
    }

    if ($SkipLlamaCpp) {
        Write-Warning "Skipping llama.cpp build (--SkipLlamaCpp)"
        return
    }

    Write-Info "Building llama.cpp with server support..."

    $llamaDir = "$env:TEMP\llama.cpp"
    if (Test-Path $llamaDir) {
        if ($ForceRebuild) {
            Write-Info "Cleaning previous build..."
            Remove-Item $llamaDir -Recurse -Force
        } else {
            Write-Info "Using existing llama.cpp source..."
        }
    }

    if (-not (Test-Path $llamaDir)) {
        Write-Info "Cloning llama.cpp..."
        git clone https://github.com/ggml-org/llama.cpp $llamaDir
        if ($LASTEXITCODE -ne 0) {
            Write-ErrorMsg "git clone of llama.cpp failed (exit code $LASTEXITCODE)."
            exit 1
        }
    }

    Set-Location $llamaDir

    Write-Info "Configuring CMake (this may take a minute)..."
    try {
        # Static libs: avoids missing ggml/llama DLLs next to llama-server.exe. CURL off: not needed, and breaks the configure step when libcurl is absent.
        cmake -B build -DLLAMA_BUILD_SERVER=ON -DLLAMA_CURL=OFF -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release
        # Native commands do not throw on failure in PowerShell, so check the exit code.
        if ($LASTEXITCODE -ne 0) { throw "cmake configure exited with code $LASTEXITCODE" }
    } catch {
        Write-ErrorMsg "CMake configure failed. Ensure CMake and Visual Studio Build Tools are installed."
        Write-Info "Install: Visual Studio 2022 Community with 'Desktop development with C++' workload"
        exit 1
    }

    Write-Info "Building llama-server (this may take 5-15 minutes)..."
    try {
        cmake --build build --config Release --parallel
        if ($LASTEXITCODE -ne 0) { throw "cmake build exited with code $LASTEXITCODE" }
    } catch {
        Write-ErrorMsg "Build failed: $_"
        exit 1
    }

    # Copy binary to project
    $builtExe = "$llamaDir\build\bin\Release\llama-server.exe"
    if (-not (Test-Path $builtExe)) {
        $builtExe = "$llamaDir\build\bin\llama-server.exe"
    }

    if (Test-Path $builtExe) {
        $targetDir = "$PSScriptRoot\src-tauri\resources"
        New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
        Copy-Item $builtExe -Destination "$targetDir\llama-server.exe" -Force
        # Copy any runtime DLLs the build produced next to the exe.
        Get-ChildItem (Split-Path $builtExe) -Filter *.dll -ErrorAction SilentlyContinue | Copy-Item -Destination $targetDir -Force
        Write-Success "llama-server.exe copied to $targetDir"
    } else {
        Write-ErrorMsg "Built binary not found at expected location"
        exit 1
    }

    Set-Location $PSScriptRoot
}

function Create-PlaceholderIcons {
    Write-Info "Creating placeholder icons..."

    $iconsDir = "$PSScriptRoot\src-tauri\icons"
    New-Item -ItemType Directory -Force -Path $iconsDir | Out-Null

    # The repository ships real icons. Never overwrite them with 1x1 placeholders:
    # that produced invalid icons and broke `tauri build` on Windows.
    $required = @('icon.ico', '32x32.png', '128x128.png', '128x128@2x.png')
    $missing = $required | Where-Object { -not (Test-Path (Join-Path $iconsDir $_)) }
    if (-not $missing) {
        Write-Success "Icons already present, keeping them"
        return
    }

    # Create a simple SVG icon
    $svg = @'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <rect width="256" height="256" rx="48" fill="#0ea5e9"/>
  <path d="M64 96h128M64 128h96M64 160h64" stroke="white" stroke-width="16" stroke-linecap="round" fill="none"/>
  <circle cx="192" cy="160" r="24" fill="white" opacity="0.2"/>
  <path d="M184 160h16M192 152v16" stroke="white" stroke-width="6" stroke-linecap="round"/>
</svg>
'@

    $svgPath = "$iconsDir\icon.svg"
    $svg | Out-File -FilePath $svgPath -Encoding utf8

    # Try to generate PNGs using ImageMagick if available
    if (Test-Command "magick") {
        Write-Info "Generating PNG icons with ImageMagick..."
        magick $svgPath -resize 32x32 "$iconsDir\32x32.png"
        magick $svgPath -resize 128x128 "$iconsDir\128x128.png"
        magick $svgPath -resize 256x256 "$iconsDir\128x128@2x.png"
        magick $svgPath -resize 512x512 "$iconsDir\icon.png"

        # Create ICO (multi-resolution)
        magick $svgPath -define icon:auto-resize=256,128,64,48,32,16 "$iconsDir\icon.ico"
        Write-Success "PNG/ICO icons generated"
    } else {
        Write-Warning "ImageMagick not found. Creating placeholder PNGs..."
        # Create minimal placeholder PNGs (1x1 transparent)
        $png1x1 = [Convert]::FromBase64String("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==")
        [IO.File]::WriteAllBytes("$iconsDir\32x32.png", $png1x1)
        [IO.File]::WriteAllBytes("$iconsDir\128x128.png", $png1x1)
        [IO.File]::WriteAllBytes("$iconsDir\128x128@2x.png", $png1x1)
        [IO.File]::WriteAllBytes("$iconsDir\icon.png", $png1x1)

        # ICO - minimal
        $icoBytes = [Convert]::FromBase64String("AAABAAEAEBAQAAEABAAoAQAAFgAAACgAAAAQAAAAIAAAAAEACAAAAAAAAAEAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAA/3QAAA==")
        [IO.File]::WriteAllBytes("$iconsDir\icon.ico", $icoBytes)

        Write-Warning "Placeholder icons created. Replace with real icons before release."
        Write-Info "Install ImageMagick: winget install ImageMagick.ImageMagick"
    }
}

function Install-NpmDependencies {
    Write-Info "Installing npm dependencies..."
    Set-Location $PSScriptRoot

    if (-not (Test-Path "package.json")) {
        Write-ErrorMsg "package.json not found in $PSScriptRoot"
        exit 1
    }

    if ($SkipNpmInstall) {
        Write-Warning "Skipping npm install (--SkipNpmInstall)"
        return
    }

    try {
        npm install
        Write-Success "npm dependencies installed"
    } catch {
        Write-ErrorMsg "npm install failed: $_"
        exit 1
    }
}

function Verify-Build {
    Write-Info "Verifying build setup..."

    $checks = @(
        @{ Name = "Rust"; Cmd = "rustc --version" },
        @{ Name = "Cargo"; Cmd = "cargo --version" },
        @{ Name = "Node"; Cmd = "node --version" },
        @{ Name = "NPM"; Cmd = "npm --version" },
        @{ Name = "Tauri CLI"; Cmd = "cargo-tauri --version" },
        @{ Name = "llama-server"; Path = "$PSScriptRoot\src-tauri\resources\llama-server.exe" },
        @{ Name = "package.json"; Path = "$PSScriptRoot\package.json" },
        @{ Name = "Cargo.toml"; Path = "$PSScriptRoot\src-tauri\Cargo.toml" },
        @{ Name = "tauri.conf.json"; Path = "$PSScriptRoot\src-tauri\tauri.conf.json" }
    )

    $allPassed = $true
    foreach ($check in $checks) {
        if ($check.Path) {
            if (Test-Path $check.Path) {
                Write-Success "$($check.Name): OK"
            } else {
                Write-ErrorMsg "$($check.Name): MISSING at $($check.Path)"
                $allPassed = $false
            }
        } else {
            try {
                $output = Invoke-Expression $check.Cmd
                Write-Success "$($check.Name): $output"
            } catch {
                Write-ErrorMsg "$($check.Name): NOT FOUND"
                $allPassed = $false
            }
        }
    }

    if ($allPassed) {
        Write-Success "All checks passed! Ready to run: `npm run tauri:dev`"
    } else {
        Write-ErrorMsg "Some checks failed. Review output above."
        exit 1
    }
}

# ============================================================================
# MAIN
# ============================================================================

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor $Cyan
Write-Host "║          AiMitra - Windows Setup Script                  ║" -ForegroundColor $Cyan
Write-Host "║          Local AI Code Assistant for 8GB RAM Laptops        ║" -ForegroundColor $Cyan
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor $Cyan
Write-Host ""

$startTime = Get-Date

if (-not $SkipRust) { Install-Rust }
if (-not $SkipNode) { Install-Node }
Install-TauriCLI
if (-not $SkipLlamaCpp) { Build-LlamaCpp }
Create-PlaceholderIcons
if (-not $SkipNpmInstall) { Install-NpmDependencies }
Verify-Build

$elapsed = (Get-Date) - $startTime
Write-Host ""
Write-Success "Setup completed in $($elapsed.ToString('mm\:ss'))"
Write-Host ""
Write-Info "Next steps:"
Write-Info "  1. cd $PSScriptRoot"
Write-Info "  2. npm run tauri:dev        # Development mode"
Write-Info "  3. npm run tauri:build      # Build installer (.exe)"
Write-Host ""