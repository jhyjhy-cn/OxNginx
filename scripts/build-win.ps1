# OxNginx Windows 打包脚本
# 使用方法: .\scripts\build-win.ps1
# 输出: build/ox-nginx_{version}/

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

# ============ 清理之前的产物 ============
Write-Info "清理之前的构建产物..."
$OutDir = Join-Path $RootDir "build\ox-nginx_$Version"
if (Test-Path $OutDir) {
    try {
        Remove-Item -Recurse -Force $OutDir -ErrorAction Stop
        Write-Info "已清理 build\ox-nginx_$Version"
    } catch {
        Write-Warn "无法删除旧目录，将使用新目录名"
        $OutDir = Join-Path $RootDir "build\ox-nginx_$Version_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
    }
}

$libsDir = Join-Path $RootDir "libs"

# ============ 构建前端 ============
Write-Info "正在构建前端..."
$feDir = Join-Path $RootDir "frontend"
Push-Location $feDir
pnpm run build
if ($LASTEXITCODE -ne 0) { Write-Err "前端构建失败" }
Pop-Location

# ============ 构建 backend ============
Write-Info "正在构建 backend..."
Push-Location $BackendDir

$env:CARGO_TERM_COLOR = "never"
cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Err "backend 构建失败" }

Pop-Location

# ============ 复制输出文件 ============
$BackendExe = Join-Path $BackendDir "target\release\ox-nginx.exe"

if (-not (Test-Path $BackendExe)) {
    Write-Err "构建失败，未找到 $BackendExe"
}

# 创建目录结构
$PanelDir = Join-Path $OutDir "server\panel"
$NginxOutDir = Join-Path $OutDir "server\nginx"

New-Item -ItemType Directory -Force -Path $PanelDir | Out-Null
New-Item -ItemType Directory -Force -Path $NginxOutDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutDir "backup") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutDir "ssl") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutDir "wwwlogs") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutDir "wwwroot") | Out-Null

# 复制可执行文件到 server/panel/
Copy-Item -Force $BackendExe $PanelDir
Write-Info "已复制 ox-nginx.exe -> server/panel/"

# 复制静态文件
$StaticDir = Join-Path $BackendDir "static"
if (Test-Path $StaticDir) {
    Copy-Item -Recurse -Force $StaticDir $PanelDir
    Write-Info "已复制 static/ -> server/panel/"
}

# 复制配置和数据目录（如果存在）
$ConfigsDir = Join-Path $BackendDir "configs"
$DatasDir = Join-Path $BackendDir "datas"
if (Test-Path $ConfigsDir) {
    Copy-Item -Recurse -Force $ConfigsDir $PanelDir
}
if (Test-Path $DatasDir) {
    Copy-Item -Recurse -Force $DatasDir $PanelDir
}

# 复制 libs 到 server/panel/libs/
if (Test-Path "$LibsDir\nginx\windows\nginx-1.30.3.zip") {
    $NginxLibsDir = Join-Path $PanelDir "libs\nginx"
    New-Item -ItemType Directory -Force -Path $NginxLibsDir | Out-Null
    Copy-Item -Force "$LibsDir\nginx\windows\nginx-1.30.3.zip" $NginxLibsDir
    Write-Info "已复制 nginx -> server/panel/libs/nginx/"
}

# ============ 打包 zip ============
Write-Info "正在打包 zip..."
$ZipPath = Join-Path $RootDir "build\ox-nginx_$Version.zip"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path $OutDir -DestinationPath $ZipPath -Force

# 清理临时目录
Remove-Item -Recurse -Force $OutDir

$Size = [math]::Round((Get-Item $ZipPath).Length / 1MB, 2)
Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "  构建完成！" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host ""
Write-Host "  输出: build\ox-nginx_$Version.zip  ${Size}MB" -ForegroundColor Cyan
Write-Host ""
