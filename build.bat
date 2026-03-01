@echo off
REM Wiki Builder

REM Delete out dir in current directory
rmdir /s /q "%~dp0out"


cd /d "%~dp0builder"
if "%~1"=="" (
    cargo run -- --build
) else (
    cargo run -- %*
)
