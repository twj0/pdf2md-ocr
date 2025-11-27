# Download PDFium library for Windows
# Run this script before using rust-ocr2md

$ErrorActionPreference = "Stop"

Write-Host "Downloading PDFium library..." -ForegroundColor Cyan

# PDFium release URL (bblanchon/pdfium-binaries project)
$arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
$downloadUrl = "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-win-$arch.tgz"

$tempDir = Join-Path $env:TEMP "pdfium_download"
$tgzFile = Join-Path $tempDir "pdfium.tgz"
$targetDir = $PSScriptRoot

# Create temp directory
if (Test-Path $tempDir) {
    Remove-Item $tempDir -Recurse -Force
}
New-Item -ItemType Directory -Path $tempDir | Out-Null

try {
    Write-Host "Downloading from: $downloadUrl"
    
    # Download the tgz file
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tgzFile -UseBasicParsing
    
    Write-Host "Extracting..."
    
    # Extract using tar (available in Windows 10+)
    Push-Location $tempDir
    tar -xzf $tgzFile
    Pop-Location
    
    # Find and copy pdfium.dll
    $dllPath = Get-ChildItem -Path $tempDir -Recurse -Filter "pdfium.dll" | Select-Object -First 1
    
    if ($dllPath) {
        $targetPath = Join-Path $targetDir "pdfium.dll"
        Copy-Item $dllPath.FullName -Destination $targetPath -Force
        Write-Host "Successfully installed pdfium.dll to: $targetPath" -ForegroundColor Green
        
        # Also copy to target/release if it exists
        $releaseDir = Join-Path $targetDir "target\release"
        if (Test-Path $releaseDir) {
            Copy-Item $dllPath.FullName -Destination (Join-Path $releaseDir "pdfium.dll") -Force
            Write-Host "Also copied to: $releaseDir" -ForegroundColor Green
        }
        
        # Also copy to target/debug if it exists
        $debugDir = Join-Path $targetDir "target\debug"
        if (Test-Path $debugDir) {
            Copy-Item $dllPath.FullName -Destination (Join-Path $debugDir "pdfium.dll") -Force
            Write-Host "Also copied to: $debugDir" -ForegroundColor Green
        }
    } else {
        Write-Host "Error: pdfium.dll not found in downloaded archive" -ForegroundColor Red
        exit 1
    }
    
} finally {
    # Cleanup
    if (Test-Path $tempDir) {
        Remove-Item $tempDir -Recurse -Force
    }
}

Write-Host "`nPDFium installation complete!" -ForegroundColor Green
Write-Host "You can now run: cargo build --release" -ForegroundColor Yellow
