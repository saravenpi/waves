# PowerShell script to install WAVES on Windows
$ErrorActionPreference = "Stop"

Write-Host "Building WAVES for Windows..." -ForegroundColor Green
cargo build --release

Write-Host "Installing WAVES..." -ForegroundColor Green

# Create installation directory
$InstallDir = "$env:LOCALAPPDATA\Programs\WAVES"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Copy binary
Copy-Item "target\release\waves.exe" -Destination "$InstallDir\waves.exe" -Force

# Copy logo
Copy-Item "assets\waves_logo.png" -Destination "$InstallDir\waves_logo.png" -Force

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding WAVES to PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "PATH updated. Please restart your terminal for changes to take effect." -ForegroundColor Yellow
}

# Create file associations
Write-Host "Registering file associations..." -ForegroundColor Green
$Extensions = @("mp3", "wav", "flac", "ogg", "m4a")
foreach ($ext in $Extensions) {
    $RegPath = "HKCU:\Software\Classes\.$ext"
    if (!(Test-Path $RegPath)) {
        New-Item -Path $RegPath -Force | Out-Null
    }
    Set-ItemProperty -Path $RegPath -Name "(Default)" -Value "WAVES.AudioFile"
}

$ProgIdPath = "HKCU:\Software\Classes\WAVES.AudioFile"
New-Item -Path $ProgIdPath -Force | Out-Null
New-Item -Path "$ProgIdPath\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path "$ProgIdPath\shell\open\command" -Name "(Default)" -Value "`"$InstallDir\waves.exe`" `"%1`""

Write-Host ""
Write-Host "WAVES has been installed successfully!" -ForegroundColor Green
Write-Host "Installation directory: $InstallDir" -ForegroundColor Cyan
Write-Host ""
Write-Host "Please restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
Write-Host "You can then run 'waves' from any directory." -ForegroundColor Cyan
