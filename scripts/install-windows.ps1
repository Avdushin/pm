param(
    # Куда ставить pm.exe (по умолчанию .cargo\bin, т.к. часто уже в PATH)
    [string]$InstallDir = "$env:USERPROFILE\.cargo\bin"
)

$ErrorActionPreference = "Stop"

Write-Host "==> pm Windows installer" -ForegroundColor Cyan
Write-Host "Install directory: $InstallDir" -ForegroundColor Cyan

if (!(Test-Path $InstallDir)) {
    Write-Host "==> Creating install directory: $InstallDir" -ForegroundColor Cyan
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$binaryUrl  = "https://github.com/Avdushin/pm/releases/latest/download/pm-windows-amd64.exe"
$binaryPath = Join-Path $InstallDir "pm.exe"

Write-Host "==> Downloading pm.exe from:" -ForegroundColor Cyan
Write-Host "    $binaryUrl" -ForegroundColor DarkCyan

Invoke-WebRequest -UseBasicParsing -Uri $binaryUrl -OutFile $binaryPath

Write-Host "✅ pm.exe downloaded to $binaryPath" -ForegroundColor Green

# Добавляем InstallDir в PATH (User), если его нет
$pathUser  = [Environment]::GetEnvironmentVariable("PATH", "User")
$entries   = $pathUser -split ';' | Where-Object { $_ -ne "" }

if ($entries -notcontains $InstallDir) {
    Write-Host "==> Adding $InstallDir to user PATH" -ForegroundColor Cyan

    if ([string]::IsNullOrEmpty($pathUser)) {
        $newPath = $InstallDir
    } else {
        $newPath = "$InstallDir;$pathUser"
    }

    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "✅ PATH updated for current user." -ForegroundColor Green
    Write-Host "   Перезапусти PowerShell/Terminal, чтобы PATH обновился." -ForegroundColor Yellow
}
else {
    Write-Host "✅ $InstallDir уже есть в PATH (User)" -ForegroundColor Green
}

Write-Host ""
Write-Host "Done. Open a new PowerShell window and run:" -ForegroundColor Green
Write-Host "  pm --help" -ForegroundColor Green
