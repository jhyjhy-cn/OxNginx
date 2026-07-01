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

# 构建前端静态文件（先构建前端，失败则无需编译后端）
Write-Info "构建前端..."
$StaticDir = Join-Path $RootDir "backend\static"
$feDir = Join-Path $RootDir "frontend"

# cmd /c 执行，输出实时显示，无 job exit code 问题
cmd /c "cd /d `"$feDir`" && pnpm run build"
if ($LASTEXITCODE -ne 0) {
    Write-Error "前端构建失败"
    exit 1
}
Write-Info "前端构建完成"
Copy-Item -Recurse "$StaticDir\*" "$OutputDir\static\"

# 构建后端二进制
Write-Info "构建后端..."
$BackendBin = Join-Path $RootDir "backend\target\x86_64-unknown-linux-gnu\release\ox-nginx"

# 检测 zig 是否安装
$ZigInstalled = $false
try {
    $zigVersion = & zig version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Info "检测到 Zig $($zigVersion.Trim())"
        $ZigInstalled = $true
    }
} catch {
    $ZigInstalled = $false
}

# 检测 cargo-zigbuild 是否安装
$CargoZigbuildInstalled = $false
try {
    $null = & cargo zigbuild --help 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Info "检测到 cargo-zigbuild"
        $CargoZigbuildInstalled = $true
    }
} catch {
    $CargoZigbuildInstalled = $false
}

Push-Location (Join-Path $RootDir "backend")
if ($ZigInstalled -and $CargoZigbuildInstalled) {
    Write-Info "使用 zigbuild 交叉编译到 Linux x86_64..."

    # 检查是否缺少 Linux 目标
    $needTarget = $false
    try {
        $null = & rustup target list --installed 2>&1 | Select-String "x86_64-unknown-linux-gnu"
        if ($LASTEXITCODE -ne 0) {
            $needTarget = $true
        }
    } catch {
        $needTarget = $true
    }

    if ($needTarget) {
        Write-Info "安装 Linux 交叉编译目标..."
        & rustup target add x86_64-unknown-linux-gnu 2>&1 | Out-Null
    }

    # 后端编译：Start-Job 方式，stderr 2>$null 防止 PowerShell 把输出当错误
    $spinner = @('⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏')
    $spin = 0

    $cargoJob = Start-Job -ScriptBlock {
        param($dir)
        Set-Location $dir
        cargo zigbuild --release --target x86_64-unknown-linux-gnu 2>$null
    } -ArgumentList (Join-Path $RootDir "backend")

    while ($cargoJob.State -eq 'Running') {
        Write-Host "`r  $($spinner[$spin % $spinner.Length]) 后端编译中 (首次编译 ring 等依赖较慢)..." -NoNewline
        $spin++
        Start-Sleep -Milliseconds 300
    }

    # 丢弃 job 输出，退出码不可靠，以 binary 是否生成判断
    Receive-Job $cargoJob 2>$null | Out-Null
    Remove-Job $cargoJob -ErrorAction SilentlyContinue
    Write-Host "`r[✓] 后端编译完成" -NoNewline; Write-Host ""

    if (-not (Test-Path $BackendBin)) {
        Write-Error "后端构建失败"
        Pop-Location
        exit 1
    }
} elseif ($ZigInstalled) {
    Write-Error "Zig 已安装但缺少 cargo-zigbuild 插件"
    Write-Host "请运行: cargo install cargo-zigbuild" -ForegroundColor Cyan
    Pop-Location
    exit 1
} else {
    Write-Error "未检测到 Zig，无法交叉编译"
    Write-Host "请安装 Zig: https://ziglang.org/download/" -ForegroundColor Cyan
    Write-Host "然后安装: cargo install cargo-zigbuild" -ForegroundColor Cyan
    Pop-Location
    exit 1
}
Pop-Location

Copy-Item $BackendBin "$OutputDir\bin\ox-nginx"

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
