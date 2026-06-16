# Build-Setup.ps1 — builds the fluxid widget, then bundles it into the custom
# installer (fluxid-setup.exe).
#
# Usage:  powershell -ExecutionPolicy Bypass -File scripts\Build-Setup.ps1
# Output: dist\fluxid-setup-v<version>.exe
#
# How it works: release-build fluxid.exe, point FLUXID_PAYLOAD at it, then
# release-build fluid-setup. Its build.rs embeds the exe via include_bytes!, so
# the installer is a single self-contained file with no external payload.

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

Write-Host "=== fluxid: Build-Setup ===" -ForegroundColor Cyan

# A running widget locks fluxid.exe; stop it before building over it.
Get-Process fluxid -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# --- Step 1: release-build the widget ---
Write-Host "`n--- Step 1: cargo build -p fluid-widget --release ---"
cargo build -p fluid-widget --release
if ($LASTEXITCODE -ne 0) { throw "fluid-widget release build failed" }

$fluxid = Join-Path $root 'target\release\fluxid.exe'
if (-not (Test-Path $fluxid)) { throw "fluxid.exe not found at $fluxid" }
$fluxidMb = [math]::Round((Get-Item $fluxid).Length / 1MB, 1)
Write-Host "  fluxid.exe built ($fluxidMb MB)" -ForegroundColor Green

# --- Step 2: build the installer with the widget embedded ---
Write-Host "`n--- Step 2: cargo build -p fluid-setup --release (embedding payload) ---"
$env:FLUXID_PAYLOAD = $fluxid
# Force a rebuild so build.rs re-embeds even if nothing else changed.
cargo build -p fluid-setup --release
$buildExit = $LASTEXITCODE
Remove-Item Env:\FLUXID_PAYLOAD -ErrorAction SilentlyContinue
if ($buildExit -ne 0) { throw "fluid-setup release build failed" }

$setup = Join-Path $root 'target\release\fluxid-setup.exe'
if (-not (Test-Path $setup)) { throw "installer not found at $setup" }
$setupMb = [math]::Round((Get-Item $setup).Length / 1MB, 1)
Write-Host "  Installer built ($setupMb MB)" -ForegroundColor Green

# --- Step 3: copy to dist\ with a versioned name ---
$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version\s*=\s*"([^"]+)"' |
    Select-Object -First 1).Matches.Groups[1].Value
if (-not $version) { $version = 'dev' }

$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
$out = Join-Path $dist "fluxid-setup-v$version.exe"
Copy-Item $setup $out -Force

# SHA-256 alongside (the release flow publishes these — see distribution memory).
$hash = (Get-FileHash $out -Algorithm SHA256).Hash
"$hash  fluxid-setup-v$version.exe" | Out-File -FilePath "$out.sha256" -Encoding ascii

Write-Host "`n=== Build complete ===" -ForegroundColor Cyan
Write-Host "  Installer: $out" -ForegroundColor Yellow
Write-Host "  SHA-256:   $hash" -ForegroundColor Yellow
