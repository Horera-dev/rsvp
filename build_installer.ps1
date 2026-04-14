# build_installer.ps1

$ErrorActionPreference = "Stop"  # stop on any error

$ProjectRoot = $PSScriptRoot
$ReleaseDir  = "$ProjectRoot\release"
$OutDir      = "$ProjectRoot\installer_output"
$InnoSetup   = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
$FfmpegUrl   = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$FfmpegPath  = "$ProjectRoot\ffmpeg.exe"

Set-Location $ProjectRoot

# ─── Step 1: Build ────────────────────────────────────────────────────────────
Write-Host "`n[1/5] Building release binary..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

# ─── Step 2: Prepare release folder ──────────────────────────────────────────
Write-Host "`n[2/5] Preparing release folder..." -ForegroundColor Cyan
if (Test-Path $ReleaseDir) { Remove-Item $ReleaseDir -Recurse -Force }
New-Item -ItemType Directory -Path "$ReleaseDir\assets" | Out-Null
New-Item -ItemType Directory -Path "$ReleaseDir\out"    | Out-Null

Copy-Item "$ProjectRoot\target\release\rsvp-generator.exe" "$ReleaseDir\"
Copy-Item "$ProjectRoot\configuration.toml"                "$ReleaseDir\"
Copy-Item "$ProjectRoot\assets\*"                          "$ReleaseDir\assets\" -Recurse

# Placeholder so Inno Setup can create the out/ directory
New-Item -ItemType File -Path "$ReleaseDir\out\.gitkeep" | Out-Null

# ─── Step 3: Download ffmpeg if not already present ───────────────────────────
Write-Host "`n[3/5] Checking ffmpeg..." -ForegroundColor Cyan
if (-not (Test-Path "$FfmpegPath")) {
    Write-Host "     Downloading ffmpeg..." -ForegroundColor Yellow
    $ZipPath = "$ProjectRoot\ffmpeg.zip"
    Invoke-WebRequest -Uri $FfmpegUrl -OutFile $ZipPath
    Expand-Archive -Path $ZipPath -DestinationPath "$ProjectRoot\ffmpeg_tmp" -Force
    $FfmpegBin = Get-ChildItem "$ProjectRoot\ffmpeg_tmp" -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
    Copy-Item $FfmpegBin.FullName "$ReleaseDir\ffmpeg.exe"
    Remove-Item $ZipPath -Force
    Remove-Item "$ProjectRoot\ffmpeg_tmp" -Recurse -Force
    Write-Host "     ffmpeg downloaded." -ForegroundColor Green
} else {
    Copy-Item $FfmpegPath "$ReleaseDir\ffmpeg.exe"
    Write-Host "     ffmpeg already present, skipping download." -ForegroundColor Green
}

# ─── Step 4: Compile installer ────────────────────────────────────────────────
Write-Host "`n[4/5] Compiling Inno Setup installer..." -ForegroundColor Cyan
if (-not (Test-Path $InnoSetup)) {
    throw "Inno Setup not found at: $InnoSetup - please install it from https://jrsoftware.org/isinfo.php"
}
& $InnoSetup "$ProjectRoot\installer.iss"
if ($LASTEXITCODE -ne 0) { throw "Inno Setup compilation failed" }

# ─── Step 5: Done ─────────────────────────────────────────────────────────────
Write-Host "`n[5/5] Done!" -ForegroundColor Green
Write-Host "Installer located at: $OutDir\RSVPGenerator-Setup.exe" -ForegroundColor White