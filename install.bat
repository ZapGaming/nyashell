@echo off
echo Installing NyaShell TUI...

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

REM Check if the executable already exists
if exist "target\release\nyashell.exe" (
    echo Found existing build: target\release\nyashell.exe
) else (
    REM Check for Rust
    where cargo >nul 2>nul
    if errorlevel 1 (
        echo Rust not found. Cannot build nyashell.exe.
        echo Please install Rust from https://rustup.rs
        echo Or run: winget install Rustlang.Rust.MSVC
        echo.
        echo Alternatively, copy the pre-built nyashell.exe to target\release\
        pause
        exit /b 1
    )
    
    REM Build the executable
    echo Building NyaShell...
    cargo build --release
    if errorlevel 1 (
        echo Build failed!
        pause
        exit /b 1
    )
)

REM Create installation directory
set INSTALL_DIR=%APPDATA%\Local\NyaShell
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM Copy executable
echo Copying executable to %INSTALL_DIR%...
copy /Y "target\release\nyashell.exe" "%INSTALL_DIR%\"

REM Copy Windows Terminal profile
echo Copying Windows Terminal profile...
copy /Y "windows-terminal-profile.json" "%INSTALL_DIR%\"

REM Install profile to Windows Terminal
echo Installing Windows Terminal profile...
powershell -ExecutionPolicy Bypass -File "%~dp0install-profile.ps1"

echo.
echo ✅ NyaShell installed successfully!
echo.
echo To use: Open Windows Terminal and select "NyaShell TUI ✨" profile.
echo Or run: "%INSTALL_DIR%\nyashell.exe"
echo.
pause
