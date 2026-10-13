# PowerShell script to package clean Zapret2 distribution
$ErrorActionPreference = "Stop"

$distRoot = "dist"
$distDir = "dist\Zapret2-Windows"
$zipFile = "dist\Zapret2-Windows-Portable.zip"

Write-Host "==============================================================================" -ForegroundColor Cyan
Write-Host "                BUILDING ZAPRET2 PORTABLE RELEASE PACKAGE                     " -ForegroundColor Cyan
Write-Host "==============================================================================" -ForegroundColor Cyan

# 1. Cleanup
Write-Host "[1/5] Cleaning previous dist directory..." -ForegroundColor Yellow
if (Test-Path $distRoot) {
    Remove-Item -Path $distRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $distDir -Force | Out-Null

# 2. Build Rust release binary
Write-Host "[2/5] Compiling Rust release binary..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    throw "Cargo build failed"
}

Copy-Item "target\release\zapret2-gui.exe" -Destination "$distDir\ZapretGUI.exe" -Force
Copy-Item "target\release\zapret2-gui.exe" -Destination "dist\ZapretGUI.exe" -Force

# 3. Copy directories
Write-Host "[3/5] Copying core engine, Lua, config, lists, utils..." -ForegroundColor Yellow
$dirs = @("bin", "lua", "config", "lists", "utils")
foreach ($d in $dirs) {
    if (Test-Path $d) {
        Copy-Item -Path $d -Destination "$distDir\$d" -Recurse -Force
    }
}

# 4. Copy documentation
Write-Host "[4/5] Copying README and LICENSE..." -ForegroundColor Yellow
$files = @("README.md", "LICENSE")
foreach ($f in $files) {
    if (Test-Path $f) {
        Copy-Item -Path $f -Destination "$distDir\$f" -Force
    }
}

# 5. Compress to ZIP
Write-Host "[5/5] Creating release ZIP archive..." -ForegroundColor Yellow
Compress-Archive -Path "$distDir\*" -DestinationPath $zipFile -Force

$zipItem = Get-Item $zipFile
$sizeMb = [math]::Round($zipItem.Length / 1MB, 2)

Write-Host ""
Write-Host "==============================================================================" -ForegroundColor Green
Write-Host " [OK] Release package assembled successfully!" -ForegroundColor Green
Write-Host "  Directory : $distDir" -ForegroundColor Green
Write-Host ("  Archive   : {0} ({1} MB)" -f $zipFile, $sizeMb) -ForegroundColor Green
Write-Host "==============================================================================" -ForegroundColor Green
