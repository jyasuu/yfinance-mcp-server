@echo off
setlocal enabledelayedexpansion

if "%YFINANCE_HTTP_PORT%"=="" (
    set YFINANCE_HTTP_PORT=8080
)

echo Starting yfinance-mcp HTTP server on port %YFINANCE_HTTP_PORT%...
echo Reports will be available at http://localhost:%YFINANCE_HTTP_PORT%/reports/
echo.

.\yfinance-mcp.exe
if %ERRORLEVEL% neq 0 (
    echo.
    echo Failed to start yfinance-mcp. Make sure yfinance-mcp.exe is in this directory.
    pause
    exit /b %ERRORLEVEL%
)
