#!/usr/bin/env pwsh
$port = if ($env:YFINANCE_HTTP_PORT) { $env:YFINANCE_HTTP_PORT } else { "8080" }
$env:YFINANCE_HTTP_PORT = $port

Write-Host "Starting yfinance-mcp HTTP server on port $port..." -ForegroundColor Cyan
Write-Host "Reports will be available at http://localhost:$port/reports/" -ForegroundColor Cyan
Write-Host ""

& ".\yfinance-mcp.exe"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to start yfinance-mcp. Make sure yfinance-mcp.exe is in this directory." -ForegroundColor Red
    pause
}
