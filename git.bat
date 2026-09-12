@echo off
REM ==========================================
REM Yunji - Git Helper
REM Usage:
REM   git.bat                -> interactive menu
REM   git.bat status         -> run a single command
REM   git.bat sync "msg"     -> add all + commit + push
REM ==========================================

cd /d "%~dp0"
setlocal

REM --- Locate git executable (always resolve to full .exe path to avoid
REM     recursing into this very batch file when run from its own dir) ---
set "GIT="
for /f "delims=" %%g in ('where git.exe 2^>nul') do (
    set "GIT=%%g"
    goto :git_found
)
:git_found
if "%GIT%"=="" (
    if exist "D:\ProgramPortable\PortableGit\cmd\git.exe" (
        set "GIT=D:\ProgramPortable\PortableGit\cmd\git.exe"
    ) else (
        echo [X] git.exe not found in PATH and PortableGit not at D:\ProgramPortable\PortableGit
        pause
        exit /b 1
    )
)

REM --- If a subcommand is passed, run it directly ---
if not "%~1"=="" goto :run_subcmd

:menu
cls
echo ==========================================
echo  Yunji - Git Helper
echo ==========================================
echo  Remote:
"%GIT%" remote get-url origin 2>nul
echo.
echo  Branch:
for /f "delims=" %%b in ('"%GIT%" rev-parse --abbrev-ref HEAD 2^>nul') do echo    %%b
echo.
echo  Available actions:
echo    [1] Status        - show working tree status
echo    [2] Add all       - stage all changes
echo    [3] Commit        - commit staged changes (prompts for message)
echo    [4] Push          - push current branch to origin
echo    [5] Pull          - pull from origin
echo    [6] Log           - show recent 10 commits
echo    [7] Diff          - show unstaged changes
echo    [8] Fetch         - fetch from origin
echo    [9] Sync          - add all + commit + push (prompts for message)
echo    [10] Branches      - list local and remote branches
echo    [11] Stash         - stash current changes
echo    [12] Stash pop     - restore stashed changes
echo    [13] Remote info   - show remote URLs
echo    [0]  Exit
echo.
set /p CHOICE="Select action: "
if "%CHOICE%"=="" goto :menu
goto :opt_%CHOICE%

:opt_1
"%GIT%" status
goto :end

:opt_2
"%GIT%" add -A
echo Done.
goto :end

:opt_3
set /p MSG="Commit message: "
if "%MSG%"=="" set MSG=update
"%GIT%" commit -m "%MSG%"
goto :end

:opt_4
"%GIT%" push
goto :end

:opt_5
"%GIT%" pull
goto :end

:opt_6
"%GIT%" log --oneline -10
goto :end

:opt_7
"%GIT%" diff
goto :end

:opt_8
"%GIT%" fetch
goto :end

:opt_9
set /p MSG="Commit message: "
if "%MSG%"=="" set MSG=update
"%GIT%" add -A
"%GIT%" commit -m "%MSG%"
if errorlevel 1 (
    echo [X] Commit failed, aborting push
    goto :end
)
"%GIT%" push
goto :end

:opt_10
echo --- Local branches ---
"%GIT%" branch
echo.
echo --- Remote branches ---
"%GIT%" branch -r
goto :end

:opt_11
"%GIT%" stash push -m "auto-stash %DATE% %TIME%"
goto :end

:opt_12
"%GIT%" stash pop
goto :end

:opt_13
"%GIT%" remote -v
goto :end

:opt_0
exit /b 0

:run_subcmd
REM Pass-through mode: git.bat <subcmd> [args...]
"%GIT%" %*
goto :end

:end
echo.
pause
