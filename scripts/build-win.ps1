# OxNginx Windows 打包脚本
# 使用方法: .\scripts\build-win.ps1
# 输出: build/ox-nginx-{version}-setup.exe (NSIS 安装包)

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Version = (Get-Content (Join-Path $RootDir "VERSION") -Raw).Trim()
$BackendDir = Join-Path $RootDir "backend"

function Write-Info { param($msg) Write-Host "[✓] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Err  { param($msg) Write-Host "[✗] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "OxNginx Windows 打包脚本 (v$Version)" -ForegroundColor Cyan
Write-Host ""

# ============ 构建前端 ============
Write-Info "开始构建前端..."
$feDir = Join-Path $RootDir "frontend"
cmd /c "cd /d `"$feDir`" && pnpm run build"
if ($LASTEXITCODE -ne 0) { Write-Err "前端构建失败" }

# ============ 检查 cargo-packager ============
function Test-CargoPackager {
    cargo --list 2>&1 | Select-String -Pattern "^\\s*packager\\s*$" -Quiet
}

if (-not (Test-CargoPackager)) {
    Write-Warn "未检测到 cargo-packager，正在安装..."
    cargo install cargo-packager --locked
    if ($LASTEXITCODE -ne 0) { Write-Err "cargo-packager 安装失败" }
    Write-Info "cargo-packager 安装完成"
} else {
    Write-Info "cargo-packager 已安装"
}

# ============ 准备 nssm.exe ============
# 从版本目录拷贝 win64 版本到 libs/ 顶层，供 cargo-packager 打进 bundle
$libsDir = Join-Path $RootDir "libs"
$NssmSrc = Join-Path $libsDir "NSSM_v2.25\win64\nssm.exe"
$NssmDst = Join-Path $libsDir "nssm.exe"
if (Test-Path $NssmSrc) {
    Copy-Item -Force $NssmSrc $NssmDst
    Write-Info "已准备 nssm.exe"
} else {
    Write-Warn "未找到 $NssmSrc，将跳过服务注册"
    Write-Warn "请从 https://nssm.cc/download 下载并解压到 libs\NSSM_v2.25\win64\"
}

# ============ 检查资源文件 ============
if (-not (Test-Path "$libsDir\nginx\windows\nginx-1.30.3.zip")) {
    Write-Err "未找到 libs\nginx\windows\nginx-1.30.3.zip"
}
if (-not (Test-Path "$libsDir\nssm.exe")) {
    Write-Warn "未找到 libs\nssm.exe，将跳过服务注册"
    Write-Warn "请从 https://nssm.cc 下载并放入 libs\ 目录"
}

# ============ cargo-packager 打包 ============
Write-Info "正在使用 cargo-packager 打包 (NSIS)..."
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
    Write-Host "  打包完成！" -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  $($NsisExe.Name)  ${Size}MB" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  请以管理员身份运行安装" -ForegroundColor Yellow
    Write-Host ""
} else {
    Write-Err "未在 build\ 目录中找到 NSIS 安装包"
}