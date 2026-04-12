#!/usr/bin/env pwsh
# RustClaw Installer for Windows
$ErrorActionPreference = "Stop"

$Repo = "Adaimade/RustClaw"
$InstallDir = "$env:USERPROFILE\.local\bin"
$ConfigDir = "$env:USERPROFILE\.rustclaw"
$Target = "rustclaw-x86_64-windows"

Write-Host ""
Write-Host "  RustClaw Installer" -ForegroundColor Cyan
Write-Host "  =====================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Platform: Windows x86_64 -> $Target" -ForegroundColor Gray

# Get latest release
Write-Host "  Downloading latest release..." -ForegroundColor Yellow
$Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
$Asset = $Release.assets | Where-Object { $_.name -like "*$Target*" } | Select-Object -First 1

if (-not $Asset) {
    Write-Host "  ERROR: Could not find release for $Target" -ForegroundColor Red
    Write-Host "  Check: https://github.com/$Repo/releases" -ForegroundColor Red
    exit 1
}

$DownloadUrl = $Asset.browser_download_url
Write-Host "  URL: $DownloadUrl" -ForegroundColor Gray

# Download and extract
$TmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$ZipPath = Join-Path $TmpDir "rustclaw.zip"
Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath

# SHA256 checksum verification
$ChecksumAsset = $Release.assets | Where-Object { $_.name -eq "checksums.txt" } | Select-Object -First 1
if ($ChecksumAsset) {
    $ChecksumUrl = $ChecksumAsset.browser_download_url
    $ChecksumFile = Join-Path $TmpDir "checksums.txt"
    Invoke-WebRequest -Uri $ChecksumUrl -OutFile $ChecksumFile
    $Expected = (Get-Content $ChecksumFile | Where-Object { $_ -like "*$Target*" }) -split '\s+' | Select-Object -First 1
    if ($Expected) {
        $Actual = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()
        if ($Actual -ne $Expected) {
            Write-Host "  ERROR: Checksum verification failed!" -ForegroundColor Red
            Write-Host "    Expected: $Expected" -ForegroundColor Red
            Write-Host "    Got:      $Actual" -ForegroundColor Red
            exit 1
        }
        Write-Host "  Checksum verified" -ForegroundColor Green
    }
}

Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

# Install binary
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$ExePath = Get-ChildItem -Path $TmpDir -Filter "rustclaw.exe" -Recurse | Select-Object -First 1
if ($ExePath) {
    Copy-Item -Path $ExePath.FullName -Destination "$InstallDir\rustclaw.exe" -Force
} else {
    Write-Host "  ERROR: rustclaw.exe not found in archive" -ForegroundColor Red
    exit 1
}

Remove-Item -Recurse -Force $TmpDir
Write-Host "  Installed: $InstallDir\rustclaw.exe" -ForegroundColor Green

# Check PATH
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$UserPath", "User")
    Write-Host "  Added $InstallDir to user PATH" -ForegroundColor Green
    Write-Host "  (Restart your terminal for PATH to take effect)" -ForegroundColor Yellow
} else {
    Write-Host "  PATH already includes $InstallDir" -ForegroundColor Green
}

# Create default config
if (-not (Test-Path "$ConfigDir\config.toml")) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    @"
[gateway]
port = 18789
bind = "127.0.0.1"
token = ""

[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen3-coder:30b"
system_prompt = "You are a helpful assistant."

[channels.telegram]
enabled = false
bot_token = ""

[channels.discord]
enabled = false
bot_token = ""
"@ | Set-Content -Path "$ConfigDir\config.toml" -Encoding UTF8
    Write-Host "  Config created: $ConfigDir\config.toml" -ForegroundColor Green
} else {
    Write-Host "  Config exists: $ConfigDir\config.toml (not overwritten)" -ForegroundColor Green
}

# Check Ollama
Write-Host ""
$OllamaPath = Get-Command ollama -ErrorAction SilentlyContinue
if ($OllamaPath) {
    Write-Host "  Ollama detected" -ForegroundColor Green
    try {
        $null = Invoke-RestMethod -Uri "http://localhost:11434/api/tags" -TimeoutSec 2
        Write-Host "  Ollama is running" -ForegroundColor Green
    } catch {
        Write-Host "  Ollama installed but not running. Start it: ollama serve" -ForegroundColor Yellow
    }
} else {
    Write-Host "  Ollama not found. Install it for local LLM:" -ForegroundColor Yellow
    Write-Host "    https://ollama.com/download" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Or use a cloud API -- edit $ConfigDir\config.toml:" -ForegroundColor Yellow
    Write-Host "    provider = `"anthropic`"" -ForegroundColor White
    Write-Host "    api_key = `"sk-ant-...`"" -ForegroundColor White
}

Write-Host ""
Write-Host "  Done! Try:" -ForegroundColor Green
Write-Host "    rustclaw agent `"Hello, what can you do?`"" -ForegroundColor White
Write-Host "    rustclaw gateway   # Start server with Telegram/Discord" -ForegroundColor White
Write-Host ""
