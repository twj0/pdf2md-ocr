# Build and package RustOCR2md for release
# This script creates a complete release package

$ErrorActionPreference = "Stop"

Write-Host "Building RustOCR2md Release Package" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan

$projectDir = $PSScriptRoot
$releaseDir = Join-Path $projectDir "target\release"
$packageDir = Join-Path $projectDir "release_package"

# Step 1: Download PDFium if not exists
$pdfiumDll = Join-Path $projectDir "pdfium.dll"
if (-not (Test-Path $pdfiumDll)) {
    Write-Host "`nStep 1: Downloading PDFium..." -ForegroundColor Yellow
    & powershell -ExecutionPolicy Bypass -File (Join-Path $projectDir "download_pdfium.ps1")
} else {
    Write-Host "`nStep 1: PDFium already exists, skipping download" -ForegroundColor Green
}

# Step 2: Build release
Write-Host "`nStep 2: Building release..." -ForegroundColor Yellow
Push-Location $projectDir
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
Pop-Location

# Step 3: Create package directory
Write-Host "`nStep 3: Creating release package..." -ForegroundColor Yellow
if (Test-Path $packageDir) {
    Remove-Item $packageDir -Recurse -Force
}
New-Item -ItemType Directory -Path $packageDir | Out-Null

# Step 4: Copy files to package
$exePath = Join-Path $releaseDir "rust-ocr2md.exe"
Copy-Item $exePath -Destination $packageDir
Copy-Item $pdfiumDll -Destination $packageDir

# Step 5: Create README
$readmeContent = @"
# RustOCR2md - PDF to Markdown OCR Tool

## Usage

### Method 1: Drag and Drop
Simply drag a PDF file onto `rust-ocr2md.exe`

### Method 2: Command Line
```
rust-ocr2md.exe <pdf_file>
rust-ocr2md.exe input.pdf -o output.md
rust-ocr2md.exe input.pdf --dpi 300 --languages eng+chi_sim
```

### Options
- `-o, --output`: Output markdown file path
- `-d, --dpi`: DPI for PDF rendering (default: 300)
- `-l, --languages`: OCR languages (default: eng+chi_sim+equ)
- `-t, --threads`: Number of threads
- `--pages`: Page range (e.g., 1-10, or "all")

## Requirements

### Tesseract Language Data
Download language data files and set `TESSDATA_PREFIX` environment variable:

1. Download from: https://github.com/tesseract-ocr/tessdata
2. Required files: `eng.traineddata`, `chi_sim.traineddata`, `equ.traineddata`
3. Place in a folder and set environment variable:
   ```
   setx TESSDATA_PREFIX "C:\path\to\tessdata"
   ```

Or place in default location:
- Windows: `%APPDATA%\tesseract-rs\tessdata\`

## Files in this package
- `rust-ocr2md.exe` - Main executable
- `pdfium.dll` - PDF rendering library
- `README.md` - This file
"@

$readmeContent | Out-File -FilePath (Join-Path $packageDir "README.md") -Encoding UTF8

Write-Host "`nRelease package created at: $packageDir" -ForegroundColor Green
Write-Host "`nContents:" -ForegroundColor Cyan
Get-ChildItem $packageDir | ForEach-Object {
    Write-Host "  - $($_.Name) ($([math]::Round($_.Length / 1MB, 2)) MB)"
}

Write-Host "`nDone! You can now distribute the release_package folder." -ForegroundColor Green
