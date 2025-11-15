@echo off

where cross >nul 2>nul

if %errorlevel% neq 0 (    
    cargo install cross
) else (
)

pause