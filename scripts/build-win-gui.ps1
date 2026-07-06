# OxNginx GUI 打包脚本
# 使用方法: .\scripts\build-win-gui.ps1 [-Sign]
# -Sign: 使用 updater.key 签名，生成 latest.json 和签名文件

param(
    [switch]$Sign
)

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Version = (Get-Content (Join-Path $RootDir "VERSION") -Raw).Trim()
$GuiDir = Join-Path $RootDir "backend-gui"
$BuildDir = Join-Path $RootDir "build"
$KeyFile = Join-Path $GuiDir "updater.key"

function Write-Info { param($msg) Write-Host "[✓] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Err  { param($msg) Write-Host "[✗] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "OxNginx GUI 打包脚本 (v$Version)" -ForegroundColor Cyan
Write-Host ""

# ============ 检查 tauri-cli ============
function Test-TauriCli {
    try {
        $null = cargo tauri --version 2>&1
        return $LASTEXITCODE -eq 0
    } catch {
        return $false
    }
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

# 先解压 backend zip 到 backend-gui/bundle 目录
$BackendZip = Join-Path $BuildDir "ox-nginx_$Version.zip"
$BundleDir = Join-Path $GuiDir "bundle"

if (Test-Path $BundleDir) {
    Remove-Item -Recurse -Force $BundleDir
}

if (Test-Path $BackendZip) {
    Expand-Archive -Path $BackendZip -DestinationPath $BundleDir -Force
    # zip 里有一层 ox-nginx_xxx 目录，把内容提到 bundle 根
    $InnerDir = Get-ChildItem -Path $BundleDir -Directory | Select-Object -First 1
    if ($InnerDir) {
        Copy-Item -Recurse -Force "$($InnerDir.FullName)\*" $BundleDir
        Remove-Item -Recurse -Force $InnerDir.FullName
    }
    Write-Info "已解压 backend 到 bundle 目录"
} else {
    Write-Err "未找到 $BackendZip，请先运行 build-win.ps1"
}

Push-Location $GuiDir

$env:CARGO_TERM_COLOR = "never"

# 签名模式：传入密钥内容，密码由 Tauri 交互读取
if ($Sign) {
    if (-not (Test-Path $KeyFile)) { Write-Err "未找到签名密钥: $KeyFile" }
    $env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content $KeyFile -Raw).Trim()
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $null
    Write-Info "签名模式已启用，请在打包时输入密钥密码"
}

cargo tauri build
if ($LASTEXITCODE -ne 0) { Write-Err "GUI 构建失败" }

# 清理签名环境变量
$env:TAURI_SIGNING_PRIVATE_KEY = $null
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = $null
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $null

Pop-Location

# ============ 步骤3: 整合输出安装包 ============
Write-Info "步骤3/3: 整合安装包..."

$GuiNsisExe = Get-ChildItem -Path "$GuiDir\target\release\bundle\nsis\*setup*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($GuiNsisExe) {
    $SetupExe = Join-Path $BuildDir "ox-nginx-gui_$Version`_setup.exe"
    Copy-Item -Force $GuiNsisExe.FullName $SetupExe
    Write-Info "已生成安装包: ox-nginx-gui_$Version`_setup.exe"
} else {
    Write-Err "未找到 NSIS 安装包"
}

# 清理中间产物
Remove-Item -Recurse -Force $BundleDir -ErrorAction SilentlyContinue
Remove-Item -Force (Join-Path $BuildDir "ox-nginx_$Version.zip") -ErrorAction SilentlyContinue

# ============ 签名产物 ============
if ($Sign) {
    # 复制 updater 包（.nsis.zip + .sig）
    $NsisZip = Get-ChildItem -Path "$GuiDir\target\release\bundle\nsis\*setup*.nsis.zip" -ErrorAction SilentlyContinue | Select-Object -First 1
    $NsisSig = Get-ChildItem -Path "$GuiDir\target\release\bundle\nsis\*setup*.nsis.zip.sig" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($NsisZip) { Copy-Item -Force $NsisZip.FullName $BuildDir }
    if ($NsisSig) { Copy-Item -Force $NsisSig.FullName $BuildDir }

    # 读取签名
    $SigContent = if ($NsisSig) { (Get-Content $NsisSig.FullName -Raw).Trim() } else { "" }
    $ZipFileName = if ($NsisZip) { $NsisZip.Name } else { "ox-nginx-gui_${Version}_x64-setup.nsis.zip" }
    $DownloadUrl = "https://github.com/jhyjhy-cn/OxNginx/releases/download/v$Version/$ZipFileName"

    # 生成 latest.json
    $LatestJson = @{
        version  = $Version
        notes    = "OxNginx v$Version"
        pub_date = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
        platforms = @{
            "windows-x86_64" = @{
                signature = $SigContent
                url       = $DownloadUrl
            }
        }
    } | ConvertTo-Json -Depth 4
    $LatestJson | Set-Content -Path (Join-Path $BuildDir "latest.json") -Encoding UTF8
    Write-Info "已生成签名产物和 latest.json"
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
    if ($Sign) {
        Write-Host "  签名包: build\ox-nginx-gui_$Version*_x64-setup.nsis.zip" -ForegroundColor Cyan
        Write-Host "  更新清单: build\latest.json" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "  发布时上传这3个文件到 GitHub Release" -ForegroundColor Yellow
    }
    Write-Host ""
} else {
    Write-Err "打包失败"
}
