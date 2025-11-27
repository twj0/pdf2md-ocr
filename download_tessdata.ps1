# Download Tesseract language data files
# Required for OCR functionality

$ErrorActionPreference = "Stop"

Write-Host "Downloading Tesseract Language Data..." -ForegroundColor Cyan

# Target directory - use APPDATA for Windows
$tessdataDir = Join-Path $env:APPDATA "tesseract-rs\tessdata"

# Create directory if it doesn't exist
if (-not (Test-Path $tessdataDir)) {
    New-Item -ItemType Directory -Path $tessdataDir -Force | Out-Null
}

Write-Host "Tessdata directory: $tessdataDir" -ForegroundColor Yellow

# Language files to download
$languages = @("eng", "chi_sim", "equ")
$baseUrl = "https://github.com/tesseract-ocr/tessdata/raw/main"

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

foreach ($lang in $languages) {
    $fileName = "$lang.traineddata"
    $targetPath = Join-Path $tessdataDir $fileName
    
    if (Test-Path $targetPath) {
        Write-Host "  $fileName already exists, skipping" -ForegroundColor Green
        continue
    }
    
    $downloadUrl = "$baseUrl/$fileName"
    Write-Host "  Downloading $fileName..." -ForegroundColor White
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $targetPath -UseBasicParsing
        Write-Host "    Downloaded successfully" -ForegroundColor Green
    } catch {
        Write-Host "    Failed to download: $_" -ForegroundColor Red
    }
}

Write-Host "`nTessdata installation complete!" -ForegroundColor Green
Write-Host "Language files installed to: $tessdataDir" -ForegroundColor Yellow

# Set environment variable for current session
$env:TESSDATA_PREFIX = $tessdataDir
Write-Host "`nTESSDATA_PREFIX set for current session: $tessdataDir" -ForegroundColor Cyan

# Prompt to set permanently
Write-Host "`nTo set TESSDATA_PREFIX permanently, run:" -ForegroundColor Yellow
Write-Host "  setx TESSDATA_PREFIX `"$tessdataDir`"" -ForegroundColor White
