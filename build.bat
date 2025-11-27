@echo off
echo Building RustOCR2md in release mode...
cargo build --release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✅ Build successful!
    echo 📦 Executable location: target\release\rust-ocr2md.exe
    echo.
    echo To run:
    echo   target\release\rust-ocr2md.exe -i input.pdf -o output.md
) else (
    echo.
    echo ❌ Build failed!
    exit /b 1
)
