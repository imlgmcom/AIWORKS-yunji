@echo off
REM ==========================================
REM Yunji Tauri Portable Build Script
REM ==========================================

cd /d "%~dp0"

set RELEASE_DIR=release\Yunji
set TARGET_EXE=src-tauri\target\release\yunji-tauri.exe

echo ==========================================
echo  Yunji Tauri Portable Build
echo ==========================================
echo.

REM --- Environment check ---
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

REM --- Generate version: YY.M.D (no leading zeros, valid semver) ---
for /f "delims=" %%v in ('powershell -NoProfile -Command "Write-Output (Get-Date -Format 'yy.M.d')"') do set APP_VERSION=%%v
echo [i] Version: %APP_VERSION%

REM Use Node.js to update version in all config files (avoids cmd.exe quoting issues)
node -e "var fs=require('fs'),v=process.env.APP_VERSION,d=String.fromCharCode(34);['package.json','src-tauri/tauri.conf.json'].forEach(function(f){var c=fs.readFileSync(f,'utf8');fs.writeFileSync(f,c.replace(new RegExp(d+'version'+d+':\\s*'+d+'[^'+d+']*'+d,'g'),d+'version'+d+': '+d+v+d))});var c=fs.readFileSync('src-tauri/Cargo.toml','utf8');fs.writeFileSync('src-tauri/Cargo.toml',c.replace(new RegExp('^version\\s*=\\s*'+d+'[^'+d+']*'+d,'m'),'version = '+d+v+d))"

echo     Version numbers updated
echo.

REM --- Build mode selection ---
echo Select build mode:
echo   [1] Full build  - clean rebuild (LTO optimization, smaller exe, slower)
echo   [2] Quick build - incremental (skip clean, faster, exe slightly larger)
echo   [3] Icon only   - regenerate app icons from icon.svg, no build
echo.
set /p BUILD_CHOICE="Enter choice (1/2/3, default 2): "
if "%BUILD_CHOICE%"=="" set BUILD_CHOICE=2

if "%BUILD_CHOICE%"=="3" (
    echo [i] Icon generation: regenerating from icon.svg...
    if not exist "icon.svg" (
        echo [X] icon.svg not found in project root
        pause
        exit /b 1
    )
    call npx tauri icon "icon.svg"
    if errorlevel 1 (
        echo [X] Icon generation failed
        pause
        exit /b 1
    )
    echo.
    echo ==========================================
    echo  Icons regenerated successfully!
    echo ==========================================
    pause
    exit /b 0
)

if "%BUILD_CHOICE%"=="1" (
    set FULL_BUILD=1
    echo [i] Full build: will clean target before compiling
) else (
    set FULL_BUILD=0
    echo [i] Quick build: incremental compilation
)
echo.

REM --- Step 1: Frontend ---
echo [1/4] Building frontend (Vite + TypeScript)...
call npm run build
if errorlevel 1 (
    echo [X] Frontend build failed
    pause
    exit /b 1
)
echo     Frontend build complete
echo.

REM --- Step 2: Rust backend ---
if "%FULL_BUILD%"=="1" (
    echo [2/4] Cleaning Rust target for full rebuild...
    cd src-tauri
    cargo clean
    cd ..
)

echo [2/4] Building Rust backend (release mode, may take a while)...
call npm run tauri build -- --no-bundle
if errorlevel 1 (
    echo [X] Rust build failed
    pause
    exit /b 1
)
echo     Rust build complete
echo.

REM --- Step 3: Prepare release dir ---
echo [3/4] Preparing portable release directory...
REM Preserve existing data\ dir (user database/uploads), only clean others
if exist "%RELEASE_DIR%\Yunji.exe" del /q "%RELEASE_DIR%\Yunji.exe" >nul 2>nul
if exist "%RELEASE_DIR%\icons" rmdir /s /q "%RELEASE_DIR%\icons" >nul 2>nul
if exist "%RELEASE_DIR%\tools" rmdir /s /q "%RELEASE_DIR%\tools" >nul 2>nul
if exist "%RELEASE_DIR%\vditor" rmdir /s /q "%RELEASE_DIR%\vditor" >nul 2>nul
if not exist "%RELEASE_DIR%" mkdir "%RELEASE_DIR%"
if not exist "%RELEASE_DIR%\data" mkdir "%RELEASE_DIR%\data"
echo     Directory ready (data\ preserved)
echo.

REM --- Step 4: Copy files ---
echo [4/4] Copying files...
if not exist "%TARGET_EXE%" (
    echo [X] Build artifact not found: %TARGET_EXE%
    echo     Check src-tauri\Cargo.toml [package] name field
    pause
    exit /b 1
)
copy "%TARGET_EXE%" "%RELEASE_DIR%\Yunji.exe" >nul
echo     Yunji.exe copied

REM Vditor resources are bundled into exe via tauri-codegen-assets, no external copy needed

if exist "src-tauri\icons" (
    if not exist "%RELEASE_DIR%\icons" mkdir "%RELEASE_DIR%\icons"
    xcopy "src-tauri\icons\*.png" "%RELEASE_DIR%\icons\" /y >nul 2>nul
    echo     icons\ copied
)

REM --- aria2c ---
echo [5/5] Downloading aria2c for magnet link support...
if not exist "tools\aria2c.exe" (
    mkdir "tools" 2>nul
    powershell -NoProfile -Command "$url='https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip'; $zip='tools\aria2.zip'; Write-Host '    Downloading aria2c...'; Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing; Expand-Archive -Path $zip -DestinationPath 'tools\aria2_temp' -Force; Copy-Item 'tools\aria2_temp\aria2-1.37.0-win-64bit-build1\aria2c.exe' 'tools\aria2c.exe' -Force; Remove-Item 'tools\aria2_temp' -Recurse -Force; Remove-Item $zip -Force; Write-Host '    aria2c.exe downloaded'"
    if errorlevel 1 (
        echo [W] aria2c download failed, magnet link feature will not work
        echo     You can manually download aria2c.exe to tools/ folder
    )
) else (
    echo     aria2c.exe already exists, skip download
)
if exist "tools\aria2c.exe" (
    if not exist "%RELEASE_DIR%\tools" mkdir "%RELEASE_DIR%\tools"
    copy "tools\aria2c.exe" "%RELEASE_DIR%\tools\aria2c.exe" >nul
    echo     tools\aria2c.exe copied
)
echo.

echo ==========================================
echo  Build Success!
echo ==========================================
echo.
echo  Release: %~dp0%RELEASE_DIR%\
echo.
echo  Usage:
echo    1. Copy release\Yunji folder to USB drive
echo    2. Double click Yunji.exe to run
echo    3. Data saved in data\ next to exe
echo.
pause
