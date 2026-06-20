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

# Download binary if missing
if (-not (Test-Path $binaryPath)) {
    if ($Version -eq "latest") {
        Write-Host "Fetching latest release from $Repo ..." -ForegroundColor Yellow
        $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest" -Headers @{"Accept" = "application/json"}
        $tag = $release.tag_name
    } else {
        $tag = if ($Version -match "^v") { $Version } else { "v$Version" }
    }
    $zipUrl = "https://github.com/$Repo/releases/download/$tag/$zipName"
    $zipPath = Join-Path $env:TEMP $zipName
    Write-Host "Downloading $zipName from $tag ..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath
    Write-Host "Extracting to $scriptDir ..." -ForegroundColor Yellow
    Expand-Archive -Path $zipPath -DestinationPath $scriptDir -Force
    Remove-Item $zipPath
    Write-Host "Installed $binaryName" -ForegroundColor Green
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
