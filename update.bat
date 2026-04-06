@echo off
echo.
echo   ========================================
echo     CrustBrowser Updater
echo   ========================================
echo.

echo [1/3] Pulling latest changes...
git pull origin main
if %errorlevel% neq 0 (
    echo [ERROR] Git pull failed. Make sure you're in the CrustBrowser directory.
    pause
    exit /b 1
)

echo [2/3] Rebuilding Rust binary...
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Build failed.
    pause
    exit /b 1
)

echo [3/3] Updating CLI dependencies...
cd CLI
call npm install
cd ..

echo.
echo   Update complete! Run with: node CLI/index.js
pause
