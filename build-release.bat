@echo off
setlocal enabledelayedexpansion

rem Build and package script for thrushc
rem Usage: build-release.bat [debug|release]

set "build_mode=release"
set "target_dir=target"
set "binary_name=thrushc.exe"
set "dist_dir=dist\windows"

if not "%~1"=="" (
    set "build_mode=%~1"
)

if not "%build_mode%"=="debug" if not "%build_mode%"=="release" (
    echo Error: Invalid build mode '%build_mode%'. Use 'debug' or 'release'.
    exit /b 1
)

if not exist "Cargo.toml" (
    echo Error: Cargo.toml not found. Run this script from the project root.
    exit /b 1
)

where upx >nul 2>&1
if errorlevel 1 (
    echo Error: UPX not found. Please install UPX for binary compression.
    exit /b 1
)

echo Building project in %build_mode% mode...

if "%build_mode%"=="release" (
    cargo build --release
    set "binary_path=%target_dir%\release\%binary_name%"
) else (
    cargo build
    set "binary_path=%target_dir%\debug\%binary_name%"
)

if errorlevel 1 (
    echo Error: Build failed.
    exit /b 1
)

if not exist "%binary_path%" (
    echo Error: Binary '%binary_name%' not found at %binary_path%
    exit /b 1
)

echo Build completed successfully.

if not exist "%dist_dir%" mkdir "%dist_dir%"

set "dist_binary=%dist_dir%\%binary_name%"
copy "%binary_path%" "%dist_binary%" >nul

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