# OxNginx Windows 打包脚本
# 使用方法: .\scripts\build-win.ps1
# 输出: build/ox-nginx-{version}-setup.exe (NSIS installer)

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Version = (Get-Content (Join-Path $RootDir "VERSION") -Raw).Trim()
$BackendDir = Join-Path $RootDir "backend"

function Write-Info { param($msg) Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[!!] $msg" -ForegroundColor Yellow }
function Write-Err  { param($msg) Write-Host "[ERR] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "OxNginx Windows Packager (v$Version)" -ForegroundColor Cyan
Write-Host ""

# ============ 构建前端 ============
Write-Info "Build frontend..."
$feDir = Join-Path $RootDir "frontend"
cmd /c "cd /d `"$feDir`" && pnpm run build"
if ($LASTEXITCODE -ne 0) { Write-Err "Frontend build failed" }

# ============ 检查资源文件 ============
$libsDir = Join-Path $RootDir "libs"
if (-not (Test-Path "$libsDir\nginx-1.30.3.zip")) {
    Write-Err "libs\nginx-1.30.3.zip not found"
}
if (-not (Test-Path "$libsDir\nssm.exe")) {
    Write-Warn "libs\nssm.exe not found - service registration will be skipped"
    Write-Warn "Download from https://nssm.cc and place in libs/"
}

# ============ cargo-packager 打包 ============
Write-Info "Packaging with cargo-packager (NSIS)..."
Push-Location $BackendDir

cargo packager --release --formats nsis 2>&1 | ForEach-Object {
    if ($_ -match 'error') { Write-Host $_ -ForegroundColor Red }
    else { Write-Host $_ }
}

Pop-Location

# ============ 检查输出 ============
$OutDir = Join-Path $RootDir "build"
$NsisExe = Get-ChildItem -Path $OutDir -Filter "*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1

if ($NsisExe) {
    $Size = [math]::Round($NsisExe.Length / 1MB, 2)
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host "  Done!" -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  $($NsisExe.Name)  ${Size}MB" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Run as Administrator to install" -ForegroundColor Yellow
    Write-Host ""
} else {
    Write-Err "NSIS package not found in build/"
}
