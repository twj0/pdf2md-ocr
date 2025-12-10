# Build and package RustOCR2md for release
# This script creates a complete release package

$ErrorActionPreference = "Stop"

Write-Host "Building RustOCR2md Release Package (v0.1.1)" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan

$projectDir = $PSScriptRoot
$version = "0.1.1"
$releaseDir = Join-Path $projectDir "target\release"
$packageDir = Join-Path $projectDir "release_package"
$versionDir = Join-Path $packageDir $version

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
# Try to stop any running instance to release file locks
Get-Process -Name "rust-ocr2md" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 200
if (Test-Path $versionDir) {
    try {
        Remove-Item $versionDir -Recurse -Force -ErrorAction Stop
    } catch {
        Write-Warning "Failed to remove existing $versionDir (file lock?). Close any running rust-ocr2md.exe and rerun."
        throw
    }
}
New-Item -ItemType Directory -Path $versionDir | Out-Null

# Step 4: Copy files to package
$exePath = Join-Path $releaseDir "rust-ocr2md.exe"
Copy-Item $exePath -Destination $versionDir
Copy-Item $pdfiumDll -Destination $versionDir

# Copy PaddleOCR models if present
$paddleModels = Join-Path $projectDir "models\paddle"
if (Test-Path $paddleModels) {
    Write-Host "Copying PaddleOCR models..." -ForegroundColor Green
    # Ensure destination directory exists
    $destModelsDir = Join-Path $versionDir "models"
    if (-not (Test-Path $destModelsDir)) {
        New-Item -ItemType Directory -Path $destModelsDir | Out-Null
    }
    Copy-Item -Path $paddleModels -Destination $destModelsDir -Recurse
} else {
    # Check if models exist in root/models/paddle path (alternative location)
    $rootPaddleModels = Join-Path $projectDir "root\models\paddle"
    if (Test-Path $rootPaddleModels) {
        Write-Host "Copying PaddleOCR models from root path..." -ForegroundColor Green
        $destModelsDir = Join-Path $versionDir "models"
        if (-not (Test-Path $destModelsDir)) {
            New-Item -ItemType Directory -Path $destModelsDir | Out-Null
        }
        Copy-Item -Path $rootPaddleModels -Destination $destModelsDir -Recurse
    } else {
        Write-Host "PaddleOCR models not found at models\paddle or root\models\paddle (skip copy)" -ForegroundColor Yellow
    }
}

# Step 5: Create README
$readmeContent = @"
# RustOCR2md v$version - PDF to Markdown OCR Tool

## Usage

### Method 1: Drag and Drop
Simply drag a PDF file onto `rust-ocr2md.exe`

### Method 2: Command Line
```
rust-ocr2md.exe <pdf_file>
rust-ocr2md.exe input.pdf -o output.md
rust-ocr2md.exe input.pdf --dpi 300 --languages eng+chi_sim
rust-ocr2md.exe book.pdf --engine paddle --paddle-model-dir models/paddle --detect-language true --math-ocr true
```

### Options
- `-o, --output`: Output markdown file path
- `-d, --dpi`: DPI for PDF rendering (default: 300)
- `-l, --languages`: OCR languages (default: eng+chi_sim+equ)
- `-t, --threads`: Number of threads (Paddle/Tesseract)
- `--engine`: `paddle` (default) or `tesseract`
- `--layout`: Enable layout ordering (default: true)
- `--detect-language`: Auto language detect (default: true)
- `--math-ocr`: Enable formula detection + OCR (default: true)
- `--paddle-model-dir`: Path to Paddle ONNX models (det/cls/rec)
- `--cache`, `--cache-preprocess`, `--cache-ocr`, `--cache-dir`: Enable/adjust caching
- `--use-gpu`: Prefer GPU for preprocessing (if available)
- `--pages`: Page range (e.g., 1-10, or "all")

## Requirements

### PaddleOCR Models
Place ONNX models under `models/paddle`:
- ch_PP-OCRv4_det_infer.onnx
- ch_ppocr_mobile_v2.0_cls_infer.onnx
- ch_PP-OCRv4_rec_infer.onnx

### Tesseract Language Data (for formulas/lang-detect)
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
- `models/paddle` - PaddleOCR ONNX models (if present when packaging)
"@

$readmeContent | Out-File -FilePath (Join-Path $versionDir "README.md") -Encoding UTF8

Write-Host "`nRelease package created at: $versionDir" -ForegroundColor Green
Write-Host "`nContents:" -ForegroundColor Cyan
Get-ChildItem $versionDir | ForEach-Object {
    Write-Host "  - $($_.Name) ($([math]::Round($_.Length / 1MB, 2)) MB)"
}

Write-Host "`nDone! You can now distribute the release_package folder." -ForegroundColor Green
