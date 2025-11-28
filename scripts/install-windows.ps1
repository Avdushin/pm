param(
    [string]$InstallDir = "$env:USERPROFILE\.cargo\bin"
)

Write-Host "==> Building pm (release)..." -ForegroundColor Cyan
cargo build --release

if (!(Test-Path $InstallDir)) {
    Write-Host "==> Creating install directory: $InstallDir" -ForegroundColor Cyan
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$source = "target\release\pm.exe"
$dest = Join-Path $InstallDir "pm.exe"

if (!(Test-Path $source)) {
    Write-Host "❌ Could not find built binary at $source" -ForegroundColor Red
    exit 1
}

Copy-Item -Path $source -Destination $dest -Force

Write-Host "✅ pm.exe installed to $InstallDir" -ForegroundColor Green

$pathEntries = $env:PATH -split ';'
if (-not ($pathEntries -contains $InstallDir)) {
    Write-Host "⚠️  $InstallDir is not in PATH." -ForegroundColor Yellow
    Write-Host "   Add it via System Properties → Environment Variables, or run:" -ForegroundColor Yellow
    Write-Host "   [Environment]::SetEnvironmentVariable('PATH', `"$InstallDir;`$env:PATH`", 'User')" -ForegroundColor Yellow
} else {
    Write-Host "✅ $InstallDir is already in PATH" -ForegroundColor Green
}
