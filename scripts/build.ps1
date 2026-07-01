# OxNginx 打包脚本 (Windows PowerShell)
# 使用方法: .\scripts\build.ps1 [-Version "1.0.0"]

param(
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

# 颜色输出
function Write-Info { param($msg) Write-Host "[✓] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Error { param($msg) Write-Host "[✗] $msg" -ForegroundColor Red }

Write-Host ""
Write-Host "🚀 OxNginx 打包开始 (v$Version)" -ForegroundColor Cyan
Write-Host ""

# 项目根目录
$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BuildDir = Join-Path $RootDir "build"
$OutputName = "ox-nginx-v$Version-linux-x86_64"
$OutputDir = Join-Path $BuildDir $OutputName
$TarFile = Join-Path $BuildDir "$OutputName.tar.gz"

# 清理旧的构建
if (Test-Path $OutputDir) {
    Remove-Item -Recurse -Force $OutputDir
}
if (Test-Path $TarFile) {
    Remove-Item -Force $TarFile
}

# 创建输出目录结构
Write-Info "创建目录结构..."
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir/bin" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir/static" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir/scripts" | Out-Null

# 检查并复制后端二进制
Write-Info "复制后端文件..."
$BackendBin = Join-Path $RootDir "backend\target\x86_64-unknown-linux-gnu\release\ox-nginx"
if (-not (Test-Path $BackendBin)) {
    Write-Error "后端二进制文件不存在: $BackendBin"
    Write-Warn "请先运行: cd backend && cargo zigbuild --release --target x86_64-unknown-linux-gnu"
    exit 1
}
Copy-Item $BackendBin "$OutputDir\bin\ox-nginx"

# 检查并复制前端静态文件
Write-Info "复制前端文件..."
$StaticDir = Join-Path $RootDir "backend\static"
if (-not (Test-Path $StaticDir)) {
    Write-Error "前端静态文件不存在: $StaticDir"
    Write-Warn "请先运行: cd frontend && pnpm run build"
    exit 1
}
Copy-Item -Recurse "$StaticDir\*" "$OutputDir\static\"

# 复制部署脚本
Write-Info "复制部署脚本..."
Copy-Item "$RootDir\scripts\deploy.sh" "$OutputDir\scripts\"

# 创建版本文件
Write-Info "创建版本文件..."
$VersionInfo = @"
OxNginx v$Version
Build Time: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
Platform: linux-x86_64
"@
Set-Content -Path "$OutputDir\VERSION" -Value $VersionInfo

# 打包
Write-Info "打包中..."
# 使用 tar 命令（Windows 10 自带）
Push-Location $BuildDir
tar -czf "$OutputName.tar.gz" "$OutputName"
Pop-Location

# 清理临时目录
Remove-Item -Recurse -Force $OutputDir

# 输出结果
$FileSize = (Get-Item $TarFile).Length / 1MB
Write-Host ""
Write-Host '=========================================='
Write-Host "✅ 打包完成！" -ForegroundColor Green
Write-Host '=========================================='
Write-Host ""
Write-Host "📦 文件: build/$OutputName.tar.gz"
Write-Host "📏 大小: $([math]::Round($FileSize, 2)) MB"
Write-Host ""
Write-Host '🚀 部署命令:' -ForegroundColor Yellow
$TarFileName = $OutputName + '.tar.gz'
$DeployStep1 = '   scp build/' + $TarFileName + ' root@server:/opt/'
$DeployStep2 = '   ssh root@server'
$DeployStep3 = '   cd /opt'
$DeployStep4 = '   tar -xzf ' + $TarFileName
$DeployStep5 = '   sudo bash ' + $OutputName + '/scripts/deploy.sh'
Write-Host $DeployStep1
Write-Host $DeployStep2
Write-Host $DeployStep3
Write-Host $DeployStep4
Write-Host $DeployStep5
Write-Host ''
Write-Host '=========================================='
