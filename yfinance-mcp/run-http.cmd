@echo off
setlocal

REM Check for yfinance-mcp.exe in PATH or current directory
where yfinance-mcp.exe >nul 2>nul
if %ERRORLEVEL% equ 0 (
    if "%YFINANCE_HTTP_PORT%"=="" set YFINANCE_HTTP_PORT=8080
    echo Starting yfinance-mcp HTTP server on port %YFINANCE_HTTP_PORT%...
    yfinance-mcp.exe
    exit /b %ERRORLEVEL%
)

if exist "%~dp0yfinance-mcp.exe" (
    if "%YFINANCE_HTTP_PORT%"=="" set YFINANCE_HTTP_PORT=8080
    echo Starting yfinance-mcp HTTP server on port %YFINANCE_HTTP_PORT%...
    "%~dp0yfinance-mcp.exe"
    exit /b %ERRORLEVEL%
)

REM Not found — delegate to PowerShell for auto-install
echo yfinance-mcp.exe not found. Attempting auto-install from GitHub...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-http.ps1"
if %ERRORLEVEL% neq 0 (
    echo.
    echo Auto-install failed. Download manually from:
    echo   https://github.com/jyasuu/yfinance-mcp/releases
    echo Place yfinance-mcp.exe in this directory or add it to PATH.
    pause
    exit /b 1
)
