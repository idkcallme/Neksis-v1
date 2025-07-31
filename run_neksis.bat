@echo off
if "%1"=="" (
    echo Usage: run_neksis.bat ^<file.nx^>
    echo Example: run_neksis.bat demo.nx
    exit /b 1
)

cd neksisc
cargo run --bin neksis -- ../%1 