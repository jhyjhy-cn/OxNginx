# 版本管理脚本
# 用法: .\scripts\version.ps1 1.2.3 [-DryRun]

param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

function Write-Info { param($msg) Write-Host "[✓] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[!] $msg" -ForegroundColor Yellow }

if ($DryRun) {
    Write-Host "[DRY-RUN] 不会写入任何文件" -ForegroundColor Yellow
}

# 1. VERSION 文件
$VersionPath = Join-Path $RootDir "VERSION"
$oldVersion = Get-Content $VersionPath
if (-not $DryRun) { Set-Content -Path $VersionPath -Value $Version }
Write-Info "VERSION -> $Version"

# 2. tauri.conf.json（JSON 字段）
$ConfPath = Join-Path $RootDir "backend-gui\tauri.conf.json"
$conf = Get-Content $ConfPath -Raw
$conf = $conf -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
if (-not $DryRun) { Set-Content -Path $ConfPath -Value $conf }
Write-Info "backend-gui/tauri.conf.json -> $Version"

# 3. Cargo.toml — 只改 [package] 下第一个 version
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

# 5. frontend/package.json（JSON 字段）
$PkgPath = Join-Path $RootDir "frontend\package.json"
$pkg = Get-Content $PkgPath -Raw
$pkg = $pkg -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
if (-not $DryRun) { Set-Content -Path $PkgPath -Value $pkg }
Write-Info "frontend/package.json -> $Version"

# 6. scripts/deploy.sh 横幅里的 v<old> → v<new>
$DeployPath = Join-Path $RootDir "scripts\deploy.sh"
if ($oldVersion -eq $Version) {
    Write-Warn "deploy.sh 跳过（旧版本==新版本）"
} else {
    $deploy = Get-Content $DeployPath
    $deploy = $deploy -replace "v$([regex]::Escape($oldVersion))", "v$Version"
    if (-not $DryRun) { Set-Content -Path $DeployPath -Value $deploy }
    Write-Info "scripts/deploy.sh -> v$Version"
}

Write-Host ""
Write-Host "版本已更新为 $Version" -ForegroundColor Cyan
if ($DryRun) { Write-Host "[DRY-RUN] 完成预演" -ForegroundColor Yellow }