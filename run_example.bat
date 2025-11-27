@echo off
REM Example script to run OCR on a PDF file
REM Usage: run_example.bat <input.pdf>

if "%1"=="" (
    echo Usage: run_example.bat ^<input.pdf^>
    exit /b 1
)

set INPUT=%1
set OUTPUT=%~n1.md

echo 🚀 Running RustOCR2md...
echo Input:  %INPUT%
echo Output: %OUTPUT%
echo.

cargo run --release -- -i "%INPUT%" -o "%OUTPUT%" --threads 8 --dpi 300

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✅ Conversion completed!
    echo 📄 Output saved to: %OUTPUT%
) else (
    echo.
    echo ❌ Conversion failed!
)
