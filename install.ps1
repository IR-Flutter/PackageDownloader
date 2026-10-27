$ErrorActionPreference = "Stop"

$Repo = "IR-Flutter/PackageDownloader"
$BinName = "irflutter"
$InstallDir = "C:\tools\irflutter"

Write-Host "🚀 Installing $BinName for Windows..." -ForegroundColor Cyan

$ReleaseTag = "v1.0.0"
try {
    $Response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    if ($Response.tag_name) {
        $ReleaseTag = $Response.tag_name
    }
} catch {
    Write-Host "ℹ️ Using default release tag $ReleaseTag"
}

$DownloadUrl = "https://github.com/$Repo/releases/download/$ReleaseTag/$BinName-windows-x64.exe"
$TargetPath = "$InstallDir\$BinName.exe"

if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

Write-Host "📥 Downloading $DownloadUrl to $TargetPath ..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TargetPath

# Add to user PATH if not present
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "✅ Added $InstallDir to User PATH." -ForegroundColor Green
}

Write-Host "`n🎉 $BinName successfully installed!" -ForegroundColor Green
Write-Host "Open a new terminal and run:"
Write-Host "  irflutter list"
Write-Host "  irflutter use pub-azs"
