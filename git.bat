@echo off
chcp 65001 >nul
setlocal EnableDelayedExpansion

set GIT="D:\ProgramPortable\PortableGit\cmd\git.exe"
set CURL="D:\ProgramPortable\PortableGit\mingw64\bin\curl.exe"
set WORK_DIR=%~dp0
set GH_USER=imlgmcom
set GH_REPO=AIWORKS-yunji

title Git Toolbox - Yunji

:menu
cls
echo.
echo  ====================================================
echo            Git Toolbox - Yunji
echo  ====================================================
echo.
echo   [1] Show status          (git status)
echo   [2] Show diff            (git diff)
echo   [3] Show log             (git log)
echo   [4] Commit and push      (add + commit + push)
echo   [5] Pull updates         (git pull)
echo   [6] Show ignored files   (check gitignore)
echo   [7] Undo last commit     (reset soft)
echo   [8] Create branch        (git branch)
echo   [9] Switch branch        (git checkout)
echo   [R] Build and release to GitHub Releases
echo   [0] Exit
echo.
echo  ====================================================
echo.
set /p choice="Please select [0-9/R]: "

if "%choice%"=="1" goto status
if "%choice%"=="2" goto diff
if "%choice%"=="3" goto log
if "%choice%"=="4" goto push
if "%choice%"=="5" goto pull
if "%choice%"=="6" goto ignored
if "%choice%"=="7" goto undo
if "%choice%"=="8" goto branch
if "%choice%"=="9" goto checkout
if /i "%choice%"=="R" (
    call "%WORK_DIR%release.bat"
)
if "%choice%"=="0" exit
echo Invalid option
timeout /t 2 >nul
goto menu

:status
cls
echo.
echo  --- Current status ---
echo.
cd /d "%WORK_DIR%"
%GIT% status
echo.
pause
goto menu

:diff
cls
echo.
echo  --- Unstaged changes ---
echo.
cd /d "%WORK_DIR%"
%GIT% diff
echo.
echo  --- Staged changes ---
echo.
%GIT% diff --cached
echo.
pause
goto menu

:log
cls
echo.
echo  --- Last 20 commits ---
echo.
cd /d "%WORK_DIR%"
%GIT% log --oneline --graph -20
echo.
pause
goto menu

:push
cls
echo.
echo  --- Commit and push ---
echo.
cd /d "%WORK_DIR%"

echo Files to commit:
%GIT% status --short
echo.

set /p msg="Commit message (Enter for default): "
if "%msg%"=="" set msg=update %date% %time%

%GIT% add -A
%GIT% commit -m "%msg%"
%GIT% push
echo.
echo  --- Done ---
echo.
pause
goto menu

:pull
cls
echo.
echo  --- Pull remote updates ---
echo.
cd /d "%WORK_DIR%"
%GIT% pull
echo.
pause
goto menu

:ignored
cls
echo.
echo  --- Files ignored by .gitignore (first 30) ---
echo.
cd /d "%WORK_DIR%"
%GIT% ls-files --others --ignored --exclude-standard
echo.
pause
goto menu

:undo
cls
echo.
echo  --- Undo last commit (keep changes staged) ---
echo.
echo Last commit:
cd /d "%WORK_DIR%"
%GIT% log --oneline -1
echo.
set /p confirm="Undo? (y/N): "
if /i "%confirm%"=="y" (
    %GIT% reset --soft HEAD~1
    echo Undone, changes are kept in the staging area
) else (
    echo Canceled
)
echo.
pause
goto menu

:branch
cls
echo.
echo  --- Branch management ---
echo.
echo Current branch:
cd /d "%WORK_DIR%"
%GIT% branch --show-current
echo.
echo All branches:
%GIT% branch
echo.
set /p newbranch="New branch name (Enter to return): "
if "%newbranch%"=="" goto menu
%GIT% checkout -b "%newbranch%"
echo Created and switched to: %newbranch%
echo.
pause
goto menu

:checkout
cls
echo.
echo  --- Switch branch ---
echo.
echo Current branch:
cd /d "%WORK_DIR%"
%GIT% branch --show-current
echo.
echo All branches:
%GIT% branch
echo.
set /p target="Branch name (Enter to return): "
if "%target%"=="" goto menu
%GIT% checkout "%target%"
echo.
pause
goto menu
