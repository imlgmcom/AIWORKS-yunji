@echo off
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

where node >nul 2>nul
if errorlevel 1 (
    echo [X] Node.js not found, please install it first
    pause
    exit /b 1
)

where cargo >nul 2>nul
if errorlevel 1 (
    echo [X] Rust not found, please install it first
    pause
    exit /b 1
)

if not exist "node_modules" (
    echo [!] First run, installing dependencies...
    call npm install
    if errorlevel 1 (
        echo [X] npm install failed
        pause
        exit /b 1
    )
)

if not exist "tools\aria2c.exe" (
    echo [!] Downloading aria2c for magnet link support...
    mkdir "tools" 2>nul
    powershell -NoProfile -Command "$url='https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip'; $zip='tools\aria2.zip'; Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing; Expand-Archive -Path $zip -DestinationPath 'tools\aria2_temp' -Force; Copy-Item 'tools\aria2_temp\aria2-1.37.0-win-64bit-build1\aria2c.exe' 'tools\aria2c.exe' -Force; Remove-Item 'tools\aria2_temp' -Recurse -Force; Remove-Item $zip -Force; Write-Host '    aria2c.exe ready'"
    if errorlevel 1 (
        echo [W] aria2c download failed, magnet link feature will not work
    )
)

REM --- Build mode selection ---
echo Select dev mode:
echo   [1] Full rebuild - clean Rust target before starting dev server
echo   [2] Quick start  - skip clean, start dev server directly (default)
echo.
set /p DEV_CHOICE="Enter choice (1/2, default 2): "
if "%DEV_CHOICE%"=="" set DEV_CHOICE=2

if "%DEV_CHOICE%"=="1" (
    echo [i] Full rebuild: cleaning Rust target...
    cd src-tauri
    cargo clean
    cd ..
    echo     Target cleaned
    echo.
) else (
    echo [i] Quick start: skipping clean
    echo.
)

echo [1/1] Starting Vite + Tauri dev server...
echo     The Yunji desktop window will open automatically - use that window.
echo     NOTE: http://localhost:5173 is only loaded by the embedded WebView.
echo          Opening it in a browser shows the UI shell but NO data
echo          (Tauri backend / file dialogs are unavailable there).
echo     Press Ctrl+C to exit
echo.

call npm run tauri dev

if errorlevel 1 (
    echo.
    echo [X] Dev server failed to start
    pause
    exit /b 1
)

echo.
echo Dev server exited
pause
