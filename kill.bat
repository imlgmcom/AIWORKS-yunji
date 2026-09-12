@echo off
chcp 936 >nul
REM ==========================================
REM Yunji Tauri Dev Launcher
REM ==========================================

cd /d "%~dp0"

echo ==========================================
echo  Yunji Tauri Dev Mode
echo ==========================================
echo.

REM --- Cleanup: kill existing dev processes ---
echo [0/1] Cleaning up existing processes...

REM Kill any process listening on port 5173 (Vite dev server)
powershell -NoProfile -Command "$port=5173; $conns=Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue; if($conns){ $pids=$conns | Select-Object -ExpandProperty OwningProcess -Unique; foreach($procId in $pids){ try { Stop-Process -Id $procId -Force -ErrorAction Stop; Write-Host \"    Killed process $procId on port $port\" } catch {} } } else { Write-Host '    Port 5173 is free' }"

REM Kill any running yunji-tauri dev exe
taskkill /F /IM yunji-tauri.exe >nul 2>nul
if not errorlevel 1 (
    echo     Killed existing yunji-tauri.exe
)

REM Kill any running aria2c (from previous magnet fetches)
taskkill /F /IM aria2c.exe >nul 2>nul

echo.

pause