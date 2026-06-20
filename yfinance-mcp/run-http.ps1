#!/usr/bin/env pwsh
param(
    [string]$Version = "latest",
    [string]$Repo = "jyasuu/yfinance-mcp"
)

$ErrorActionPreference = "Stop"
$binaryName = "yfinance-mcp.exe"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$binaryPath = Join-Path $scriptDir $binaryName
$zipName = "yfinance-mcp-windows-amd64.zip"

function Download-Release {
    param([string]$downloadUrl, [string]$tag)
    $zipPath = Join-Path $env:TEMP $zipName
    Write-Host "Downloading $zipName ($tag) ..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
    Write-Host "Extracting to $scriptDir ..." -ForegroundColor Yellow
    Expand-Archive -Path $zipPath -DestinationPath $scriptDir -Force
    Remove-Item $zipPath
    Write-Host "Installed $binaryName" -ForegroundColor Green
}

# Download binary if missing
if (-not (Test-Path $binaryPath)) {
    $tag = $Version
    if ($Version -eq "latest") {
        Write-Host "Fetching latest release from $Repo ..." -ForegroundColor Yellow
        try {
            $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest" -Headers @{"Accept" = "application/json"}
            $tag = $release.tag_name
        } catch {
            Write-Host "Could not fetch latest release. Trying 'latest' redirect ..." -ForegroundColor DarkYellow
            $tag = "latest"
        }
    } elseif ($tag -notmatch "^v") {
        $tag = "v$tag"
    }

    # Try download (latest uses redirect URL, specific tags use download URL)
    $dlUrl = if ($tag -eq "latest") {
        "https://github.com/$Repo/releases/latest/download/$zipName"
    } else {
        "https://github.com/$Repo/releases/download/$tag/$zipName"
    }
    try {
        Download-Release $dlUrl $tag
    } catch {
            Write-Host "ERROR: Could not download yfinance-mcp Windows binary." -ForegroundColor Red
            Write-Host ""
            Write-Host "No releases found at https://github.com/$Repo/releases" -ForegroundColor Yellow
            Write-Host ""
            Write-Host "Options:" -ForegroundColor Cyan
            Write-Host "  1. Build from source: cargo build --release" -ForegroundColor White
            Write-Host "  2. Visit the releases page and download manually:" -ForegroundColor White
            Write-Host "     https://github.com/$Repo/releases" -ForegroundColor White
            Write-Host "  3. Place yfinance-mcp.exe manually in this directory" -ForegroundColor White
            pause
            exit 1
        }
    }

$port = if ($env:YFINANCE_HTTP_PORT) { $env:YFINANCE_HTTP_PORT } else { "8080" }
$env:YFINANCE_HTTP_PORT = $port
Write-Host "Starting yfinance-mcp HTTP server on port $port ..." -ForegroundColor Cyan
Write-Host "Reports at http://localhost:$port/reports/" -ForegroundColor Cyan
& $binaryPath
if ($LASTEXITCODE -ne 0) {
    Write-Host "yfinance-mcp exited with code $LASTEXITCODE" -ForegroundColor Red
    pause
}
