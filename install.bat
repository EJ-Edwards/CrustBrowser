@echo off
echo.
echo   ========================================
echo     CrustBrowser Installer
echo   ========================================
echo.

:: Check for Rust
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust is not installed.
    echo   Install it from: https://www.rust-lang.org/tools/install
    echo   Then run this script again.
    pause
    exit /b 1
)

:: Check for Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is not installed.
    echo   Install it from: https://nodejs.org/
    echo   Then run this script again.
    pause
    exit /b 1
)

echo [1/3] Building Rust binary...
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Rust build failed. Make sure Visual Studio Build Tools are installed.
    echo   Install from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
    pause
    exit /b 1
)

echo [2/3] Installing CLI dependencies...
cd CLI
call npm install
cd ..

echo [3/3] Setup complete!
echo.
echo   To run CrustBrowser:
echo     node CLI/index.js
echo.
echo   Or add this to your PATH for quick access.
pause
