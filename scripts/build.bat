@echo off
setlocal enabledelayedexpansion
rem Build and package script for thrushc
rem Usage: build-release.bat [debug|release]
rem This script should be run from the /scripts directory

set "build_mode=release"
set "project_root=.."
set "target_dir=%project_root%\target"
set "binary_name=thrushc.exe"
set "dist_dir=%project_root%\dist\windows"

if not "%~1"=="" (
    set "build_mode=%~1"
)

if not "%build_mode%"=="debug" if not "%build_mode%"=="release" (
    echo Error: Invalid build mode '%build_mode%'. Use 'debug' or 'release'.
    exit /b 1
)

if not exist "%project_root%\Cargo.toml" (
    echo Error: Cargo.toml not found. Make sure this script is run from the /scripts directory.
    exit /b 1
)

where upx >nul 2>&1
if errorlevel 1 (
    echo Error: UPX not found. Please install UPX for binary compression.
    exit /b 1
)

echo Building project in %build_mode% mode...

pushd "%project_root%"

if "%build_mode%"=="release" (
    cargo build --release
    set "binary_path=target\release\%binary_name%"
) else (
    cargo build
    set "binary_path=target\debug\%binary_name%"
)

set "build_result=%errorlevel%"
popd

if %build_result% neq 0 (
    echo Error: Build failed.
    exit /b 1
)

if not exist "%target_dir%\release\%binary_name%" if not exist "%target_dir%\debug\%binary_name%" (
    echo Error: Binary '%binary_name%' not found in target directory
    exit /b 1
)

echo Build completed successfully.

if not exist "%dist_dir%" mkdir "%dist_dir%"

if "%build_mode%"=="release" (
    set "source_binary=%target_dir%\release\%binary_name%"
) else (
    set "source_binary=%target_dir%\debug\%binary_name%"
)

set "dist_binary=%dist_dir%\%binary_name%"
copy "%source_binary%" "%dist_binary%" >nul

if "%build_mode%"=="release" (
    where strip >nul 2>&1
    if not errorlevel 1 (
        echo Stripping binary...
        strip "%dist_binary%"
        if not errorlevel 1 (
            echo Binary stripped successfully.
        ) else (
            echo Warning: Failed to strip binary.
        )
    ) else (
        echo Note: strip command not available. Binary will not be stripped.
    )
)

echo Compressing binary with UPX...
upx --best "%dist_binary%"
if not errorlevel 1 (
    echo Binary compressed successfully.
    
    rem Display final binary info
    for %%A in ("%dist_binary%") do set "final_size=%%~zA"
    set /a "final_size_kb=!final_size!/1024"
    echo Final binary: %dist_binary% (!final_size_kb! KB^)
    echo Build and packaging completed.
) else (
    echo Error: UPX compression failed.
    exit /b 1
)

endlocal