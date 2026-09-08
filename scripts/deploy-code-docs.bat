@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%.."
set "PROJECT_DIR=%cd%"

echo Using project directory: %PROJECT_DIR%

git rev-parse --verify origin/gh-pages >nul 2>&1
if %errorlevel% neq 0 (
    echo Branch 'gh-pages' not found. Creating...
    for /f "tokens=*" %%i in ('git branch --show-current') do set CURRENT_BRANCH=%%i
    git checkout --orphan gh-pages
    git rm -rf .
    git commit --allow-empty -m "Initial gh-pages commit"
    git push origin gh-pages
    git checkout !CURRENT_BRANCH!
)

echo Generating documentation...
cargo clean --doc
cargo docs

echo Preparing documentation...
set "TEMP_DOCS=%TEMP%\thrust-docs-build"
set "TEMP_WEBSITE_SOURCE=%TEMP%\thrust-website-source"
set "TEMP_WEBSITE_BUILD=%TEMP%\thrust-website-build"
if not defined WEBSITE_REPO_URL set "WEBSITE_REPO_URL=https://github.com/thrustlang/website"
if not defined PYTHON_BIN set "PYTHON_BIN=python"
if exist "%TEMP_DOCS%" rd /s /q "%TEMP_DOCS%"
if exist "%TEMP_WEBSITE_BUILD%" rd /s /q "%TEMP_WEBSITE_BUILD%"
xcopy /e /i /y "target\doc" "%TEMP_DOCS%"

echo ^<meta http-equiv="refresh" content="0; url=thrustc/index.html"^> > "%TEMP_DOCS%\index.html"

echo Preparing website...
if defined WEBSITE_SOURCE_DIR (
    set "WEBSITE_SOURCE=%WEBSITE_SOURCE_DIR%"
) else (
    if exist "%TEMP_WEBSITE_SOURCE%" rd /s /q "%TEMP_WEBSITE_SOURCE%"
    git clone --depth 1 "%WEBSITE_REPO_URL%" "%TEMP_WEBSITE_SOURCE%"
    set "WEBSITE_SOURCE=%TEMP_WEBSITE_SOURCE%"
)
"%PYTHON_BIN%" "%WEBSITE_SOURCE%\scripts\build_subpath.py" --source "%WEBSITE_SOURCE%" --base-path /website --output "%TEMP_WEBSITE_BUILD%"

echo Deploying to GitHub Pages...
set "PAGES_WORKTREE=%TEMP%\thrust-gh-pages"
if exist "%PAGES_WORKTREE%" rd /s /q "%PAGES_WORKTREE%"

git fetch origin gh-pages
git worktree add "%PAGES_WORKTREE%" gh-pages

pushd "%PAGES_WORKTREE%"
    for /f "delims=" %%i in ('dir /b') do (
        if not "%%i"==".git" (
            if exist "%%i\" (rd /s /q "%%i") else (del /q "%%i")
        )
    )
    type nul > ".nojekyll"
    
    xcopy /e /y "%TEMP_DOCS%\*" "."
    if exist "website" rd /s /q "website"
    xcopy /e /i /y "%TEMP_WEBSITE_BUILD%" "website"
    
    git add -A
    git diff-index --quiet HEAD --
    if %errorlevel% equ 0 (
        echo No changes to documentation.
    ) else (
        git commit -m "Update documentation %date% %time%"
        git push origin gh-pages
    )
popd

git worktree remove "%PAGES_WORKTREE%"
if exist "%TEMP_DOCS%" rd /s /q "%TEMP_DOCS%"
if exist "%TEMP_WEBSITE_SOURCE%" rd /s /q "%TEMP_WEBSITE_SOURCE%"
if exist "%TEMP_WEBSITE_BUILD%" rd /s /q "%TEMP_WEBSITE_BUILD%"
echo Done.
