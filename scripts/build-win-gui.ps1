# OxNginx GUI 打包脚本
# 使用方法: .\scripts\build-win-gui.ps1
# 输出: build/ox-nginx_{version}_setup.exe

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Version = (Get-Content (Join-Path $RootDir "VERSION") -Raw).Trim()
$GuiDir = Join-Path $RootDir "backend-gui"
$BuildDir = Join-Path $RootDir "build"

function Write-Info { param($msg) Write-Host "[✓] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Err  { param($msg) Write-Host "[✗] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "OxNginx GUI 打包脚本 (v$Version)" -ForegroundColor Cyan
Write-Host ""

# ============ 检查 tauri-cli ============
function Test-TauriCli {
    $result = cargo tauri --version 2>&1
    return $LASTEXITCODE -eq 0
}

if (-not (Test-TauriCli)) {
    Write-Warn "未检测到 tauri-cli，正在安装..."
    cargo install tauri-cli
    if ($LASTEXITCODE -ne 0) { Write-Err "tauri-cli 安装失败" }
    Write-Info "tauri-cli 安装完成"
} else {
    Write-Info "tauri-cli 已安装"
}

# ============ 步骤1: 打包 backend ============
Write-Info "步骤1/3: 打包 backend..."
$BuildScript = Join-Path $RootDir "scripts\build-win.ps1"
& $BuildScript
if ($LASTEXITCODE -ne 0) { Write-Err "backend 打包失败" }

# ============ 步骤2: 构建 GUI ============
Write-Info "步骤2/3: 构建 GUI..."

# 先复制 backend 文件到 backend-gui/bundle 目录
$BackendOutDir = Join-Path $BuildDir "ox-nginx_$Version"
$BundleDir = Join-Path $GuiDir "bundle"

if (Test-Path $BundleDir) {
    Remove-Item -Recurse -Force $BundleDir
}
New-Item -ItemType Directory -Force -Path $BundleDir | Out-Null

# 复制 backend 输出到 bundle 目录
if (Test-Path $BackendOutDir) {
    Copy-Item -Recurse -Force "$BackendOutDir\*" $BundleDir
    Write-Info "已复制 backend 文件到 bundle目录"
}

Push-Location $GuiDir

$env:CARGO_TERM_COLOR = "never"
cargo tauri build
if ($LASTEXITCODE -ne 0) { Write-Err "GUI 构建失败" }

Pop-Location

# ============ 步骤3: 整合输出安装包 ============
Write-Info "步骤3/3: 整合安装包..."

$GuiNsisExe = Get-ChildItem -Path "$GuiDir\target\release\bundle\nsis\*setup*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1

# 复制 NSIS 安装包到 build目录，重命名为标准名称
if ($GuiNsisExe) {
    $SetupExe = Join-Path $BuildDir "ox-nginx-gui_$Version`_setup.exe"
    Copy-Item -Force $GuiNsisExe.FullName $SetupExe
    Write-Info "已生成安装包: ox-nginx-gui_$Version`_setup.exe"
} else {
    Write-Warn "未找到 NSIS 安装包"
}

# 创建整合目录
$FinalDir = Join-Path $BuildDir "ox-nginx-gui_$Version"
if (Test-Path $FinalDir) {
    Remove-Item -Recurse -Force $FinalDir -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $FinalDir | Out-Null

# 复制 GUI exe
$GuiExe = Join-Path $GuiDir "target\release\ox-nginx-gui.exe"
if (Test-Path $GuiExe) {
    Copy-Item -Force $GuiExe $FinalDir
}

# 复制 backend 输出目录内容
if (Test-Path $BackendOutDir) {
    Copy-Item -Recurse -Force "$BackendOutDir\*" $FinalDir
    Write-Info "已复制 backend 文件到整合目录"
}

# ============ 检查输出 ============
$SetupExe = Join-Path $BuildDir "ox-nginx-gui_$Version`_setup.exe"
if (Test-Path $SetupExe) {
    $Size = [math]::Round((Get-Item $SetupExe).Length / 1MB, 2)
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host "  GUI 打包完成！" -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  安装包: build\ox-nginx-gui_$Version`_setup.exe  ${Size}MB" -ForegroundColor Cyan
    Write-Host ""
} elseif (Test-Path $FinalDir) {
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host "  GUI 构建完成（无安装包）" -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  输出目录: build\ox-nginx-gui_$Version" -ForegroundColor Cyan
    Write-Host ""
} else {
    Write-Err "打包失败"
}
