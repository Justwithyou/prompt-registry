@echo off
setlocal
set "LOG=%TEMP%\prompt-manager-dev.log"
cd /d "%~dp0"

>>"%LOG%" echo ===== %date% %time% =====

rem Free port 1420 & kill stale instance so strictPort won't silently fail.
for /f "tokens=5" %%P in ('netstat -ano ^| findstr ":1420 " ^| findstr "LISTENING"') do taskkill /F /PID %%P >nul 2>&1
taskkill /F /IM prompt-manager.exe >nul 2>&1

where pnpm >nul 2>nul
if errorlevel 1 (
    >>"%LOG%" echo [ERROR] pnpm not found in PATH
    echo [ERROR] pnpm not found. Install Node.js + pnpm first.
    pause
    exit /b 1
)

>>"%LOG%" echo starting: pnpm tauri dev

rem Run inside a CHILD cmd so any `exit` in the pnpm/tauri shim chain only
rem closes the child, never this console window (keeps `pause` reachable).
set "RUST_BACKTRACE=1"
set "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--enable-logging --v=1"
cmd /c "pnpm tauri dev" >> "%LOG%" 2>&1

>>"%LOG%" echo ===== exit code: %ERRORLEVEL% =====

echo.
echo prompt-manager exited (code %ERRORLEVEL%). Log: %LOG%
pause
endlocal