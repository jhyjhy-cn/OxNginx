# 版本管理脚本
# 用法: .\scripts\version.ps1 1.0.0

param(
    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

function Write-Info { param($msg) Write-Host "[✓] $msg" -ForegroundColor Green }

# 1. VERSION 文件
Set-Content -Path (Join-Path $RootDir "VERSION") -Value $Version
Write-Info "VERSION -> $Version"

# 2. tauri.conf.json
$ConfPath = Join-Path $RootDir "backend-gui\tauri.conf.json"
$conf = Get-Content $ConfPath -Raw
$conf = $conf -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
Set-Content -Path $ConfPath -Value $conf
Write-Info "tauri.conf.json -> $Version"

# 3. Cargo.toml — 只改 [package] 下第一个 version
$CargoPath = Join-Path $RootDir "backend-gui\Cargo.toml"
$cargo = Get-Content $CargoPath
$inPackage = $false
for ($i = 0; $i -lt $cargo.Count; $i++) {
    if ($cargo[$i] -match '^\[package\]') { $inPackage = $true; continue }
    if ($inPackage -and $cargo[$i] -match '^\[') { break }
    if ($inPackage -and $cargo[$i] -match '^version\s*=') {
        $cargo[$i] = "version = `"$Version`""
        break
    }
}
Set-Content -Path $CargoPath -Value $cargo
Write-Info "Cargo.toml -> $Version"

Write-Host ""
Write-Host "版本已更新为 $Version" -ForegroundColor Cyan
