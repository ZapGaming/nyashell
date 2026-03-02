@echo off
cd /d "%~dp0"
echo Building NyaShell TUI...
echo.

REM Check for Rust
where cargo >nul 2>nul
if errorlevel 1 (
    echo ERROR: Rust not found. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Build the executable
echo Compiling...
cargo build --release
if errorlevel 1 (
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo Build complete!
echo Output: target\release\nyashell.exe
echo.
echo Run install.bat to install and set up Windows Terminal profile.
echo.
pause