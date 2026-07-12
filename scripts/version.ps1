# Version bump script
# Usage: .\scripts\version.ps1 1.2.3 [-DryRun]

param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

function Write-Info { param($msg) Write-Host "[+] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[!] $msg" -ForegroundColor Yellow }

if ($DryRun) {
    Write-Host "[DRY-RUN] no files will be written" -ForegroundColor Yellow
}

# 1. VERSION file
$VersionPath = Join-Path $RootDir "VERSION"
$oldVersion = Get-Content $VersionPath
if (-not $DryRun) { Set-Content -Path $VersionPath -Value $Version }
Write-Info "VERSION -> $Version"

# 2. tauri.conf.json (JSON field)
$ConfPath = Join-Path $RootDir "backend-gui\tauri.conf.json"
$conf = Get-Content $ConfPath -Raw
$conf = $conf -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
if (-not $DryRun) { Set-Content -Path $ConfPath -Value $conf }
Write-Info "backend-gui/tauri.conf.json -> $Version"

# 3. Cargo.toml — only the first version under [package]
function Update-CargoVersion {
    param([string]$Path)
    $cargo = Get-Content $Path
    $inPackage = $false
    for ($i = 0; $i -lt $cargo.Count; $i++) {
        if ($cargo[$i] -match '^\[package\]') {
            $inPackage = $true
            continue
        }
        if ($inPackage -and $cargo[$i] -match '^\[') {
            break
        }
        if ($inPackage -and $cargo[$i] -match '^version\s*=') {
            $cargo[$i] = "version = `"$Version`""
            break
        }
    }
    if (-not $DryRun) {
        Set-Content -Path $Path -Value $cargo
    }
}

Update-CargoVersion (Join-Path $RootDir "backend-gui\Cargo.toml")
Write-Info "backend-gui/Cargo.toml -> $Version"

# 4. backend/Cargo.toml
Update-CargoVersion (Join-Path $RootDir "backend\Cargo.toml")
Write-Info "backend/Cargo.toml -> $Version"

# 5. frontend/package.json (JSON field)
$PkgPath = Join-Path $RootDir "frontend\package.json"
$pkg = Get-Content $PkgPath -Raw
$pkg = $pkg -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
if (-not $DryRun) { Set-Content -Path $PkgPath -Value $pkg }
Write-Info "frontend/package.json -> $Version"

# 6. scripts/deploy.sh banners — replace v<old> with v<new>
$DeployPath = Join-Path $RootDir "scripts\deploy.sh"
if ($oldVersion -eq $Version) {
    Write-Warn "deploy.sh skipped (old==new)"
} else {
    $deploy = Get-Content $DeployPath
    $deploy = $deploy -replace "v$([regex]::Escape($oldVersion))", "v$Version"
    if (-not $DryRun) { Set-Content -Path $DeployPath -Value $deploy }
    Write-Info "scripts/deploy.sh -> v$Version"
}

Write-Host ""
Write-Host "Version bumped to $Version" -ForegroundColor Cyan
if ($DryRun) { Write-Host "[DRY-RUN] preview complete" -ForegroundColor Yellow }