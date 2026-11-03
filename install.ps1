$ErrorActionPreference = "Stop"

$Repo = "IR-Flutter/PackageDownloader"
$BinName = "pdown"
$AliasName = "irflutter"
$InstallDir = "$env:LOCALAPPDATA\Programs\pdown"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "🚀 Installing PackageDownloader 2.0 (pdown) for Windows..." -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

$ReleaseTag = "v2.0.0"
try {
    $Response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    if ($Response.tag_name) {
        $ReleaseTag = $Response.tag_name
    }
} catch {
    Write-Host "ℹ️ Using default release tag $ReleaseTag"
}

$DownloadUrl = "https://github.com/$Repo/releases/download/$ReleaseTag/$BinName-windows-x64.exe"
$ChecksumUrl = "https://github.com/$Repo/releases/download/$ReleaseTag/checksums.txt"

if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$TempDir = [System.IO.Path]::GetTempPath()
$TempFile = Join-Path $TempDir "$BinName-download.exe"
$TempChecksum = Join-Path $TempDir "checksums.txt"

Write-Host "📥 Downloading binary from $DownloadUrl ..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile

# Verify Checksum
try {
    Invoke-WebRequest -Uri $ChecksumUrl -OutFile $TempChecksum
    $ExpectedHash = (Get-Content $TempChecksum | Select-String "$BinName-windows-x64.exe").ToString().Split(" ")[0].Trim()
    if ($ExpectedHash) {
        $ActualHash = (Get-FileHash -Path $TempFile -Algorithm SHA256).Hash.ToLower()
        if ($ExpectedHash.ToLower() -ne $ActualHash) {
            Write-Error "❌ Checksum verification failed! Expected: $ExpectedHash, Actual: $ActualHash"
            exit 1
        }
        Write-Host "✅ SHA-256 Checksum verified." -ForegroundColor Green
    }
} catch {
    Write-Host "ℹ️ Proceeding with downloaded binary."
}

$TargetPath = "$InstallDir\$BinName.exe"
$AliasPath = "$InstallDir\$AliasName.exe"

Move-Item -Force -Path $TempFile -Destination $TargetPath
Copy-Item -Force -Path $TargetPath -Destination $AliasPath

# Ensure install directory is in User PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "✅ Added $InstallDir to User PATH." -ForegroundColor Green
}

Write-Host "`n🎉 PackageDownloader 2.0 successfully installed!" -ForegroundColor Green
Write-Host "Open a new PowerShell window and run:" -ForegroundColor Yellow
Write-Host "  pdown list"
Write-Host "  pdown doctor"
Write-Host "  pdown use pub-azs"
