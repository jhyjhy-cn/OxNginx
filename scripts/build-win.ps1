# OxNginx Windows 打包脚本
# 使用方法: .\scripts\build-win.ps1
# 输出: build/ox-nginx-{version}-windows-x86_64.zip
# 部署: 解压 -> 右键管理员运行 setup.cmd

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Version = (Get-Content (Join-Path $RootDir "VERSION") -Raw).Trim()

function Write-Info { param($msg) Write-Host "[✓] $msg" -ForegroundColor Green }
function Write-Err  { param($msg) Write-Host "[✗] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "OxNginx Windows 打包 (v$Version)" -ForegroundColor Cyan
Write-Host ""

$BuildDir = Join-Path $RootDir "build"
$PkgName = "ox-nginx-v${Version}-windows-x86_64"
$PkgDir = Join-Path $BuildDir $PkgName
$ZipFile = Join-Path $BuildDir "$PkgName.zip"

if (Test-Path $PkgDir) { Remove-Item -Recurse -Force $PkgDir }
if (Test-Path $ZipFile) { Remove-Item -Force $ZipFile }

New-Item -ItemType Directory -Force -Path "$PkgDir\bin" | Out-Null
New-Item -ItemType Directory -Force -Path "$PkgDir\static" | Out-Null
New-Item -ItemType Directory -Force -Path "$PkgDir\libs" | Out-Null

# ============ 构建前端 ============
Write-Info "构建前端..."
$feDir = Join-Path $RootDir "frontend"
cmd /c "cd /d `"$feDir`" && pnpm run build"
if ($LASTEXITCODE -ne 0) { Write-Err "前端构建失败" }
Copy-Item -Recurse "$(Join-Path $RootDir 'backend\static')\*" "$PkgDir\static\"

# ============ 构建后端 ============
Write-Info "构建后端..."
Push-Location (Join-Path $RootDir "backend")
$spinner = @('⠋','⠙','⠹','⠸','⠼','⠴','⠦','⠧','⠇','⠏')
$spin = 0
$job = Start-Job { param($d); Set-Location $d; cargo build --release 2>$null } -ArgumentList (Join-Path $RootDir "backend")
while ($job.State -eq 'Running') {
    Write-Host "`r  $($spinner[$spin % $spinner.Length]) 编译中..." -NoNewline
    $spin++; Start-Sleep -Milliseconds 300
}
Receive-Job $job 2>$null | Out-Null; Remove-Job $job -EA SilentlyContinue
Write-Host "`r[✓] 后端编译完成         "
Pop-Location

$BackendExe = Join-Path $RootDir "backend\target\release\ox-nginx.exe"
if (-not (Test-Path $BackendExe)) { Write-Err "后端构建失败" }
Copy-Item $BackendExe "$PkgDir\bin\ox-nginx.exe"

# ============ 复制 nginx ============
Write-Info "复制 nginx..."
Copy-Item (Join-Path $RootDir "libs\nginx-1.30.3.zip") "$PkgDir\libs\nginx-windows.zip"

# ============ 生成 setup.cmd ============
Write-Info "生成安装脚本..."
$Setup = @"
@echo off
chcp 65001 >nul 2>&1
title OxNginx v$Version Setup

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo   [!] 请右键此文件 - 以管理员身份运行
    echo.
    pause
    exit /b 1
)

set "INST=C:\oxnginx"
set "SRC=%~dp0"

echo.
echo   OxNginx v$Version
echo   ========================================
echo.

echo   [1/5] 创建目录...
for %%d in (
    "%INST%\server\panel\configs"
    "%INST%\server\panel\datas"
    "%INST%\server\panel\static"
    "%INST%\server\nginx"
    "%INST%\wwwroot"
    "%INST%\wwwlogs"
    "%INST%\ssl"
    "%INST%\backup"
    "%INST%\tools"
) do mkdir %%d 2>nul

echo   [2/5] 复制文件...
copy /Y "%SRC%bin\ox-nginx.exe" "%INST%\server\panel\" >nul
xcopy /E /I /Y "%SRC%static" "%INST%\server\panel\static" >nul

echo   [3/5] 安装 nginx...
powershell -Command "Expand-Archive -Path '%SRC%libs\nginx-windows.zip' -DestinationPath '%INST%\server\nginx' -Force"

echo   [4/5] 生成配置...
for /f "delims=" %%i in ('powershell -Command "[Convert]::ToBase64String([byte[]](1..32|ForEach-Object{Get-Random -Maximum 256}))"') do set "JWT=%%i"

(
echo [server]
echo port = 9000
echo host = "0.0.0.0"
echo.
echo [database]
echo path = "%INST%\server\panel\datas\data.db"
echo.
echo [nginx]
echo bin = "%INST%\server\nginx\nginx.exe"
echo config = "%INST%\server\nginx\conf\nginx.conf"
echo sites_enabled = "%INST%\server\nginx\conf\sites-enabled"
echo ssl_dir = "%INST%\ssl"
echo default_root = "%INST%\wwwroot"
echo log_access = "%INST%\wwwlogs\access.log"
echo error_log = "%INST%\wwwlogs\error.log"
echo.
echo [auth]
echo jwt_secret = "%JWT%"
echo jwt_expires_hours = 24
) > "%INST%\server\panel\configs\config.toml"

mkdir "%INST%\server\nginx\conf\sites-enabled" 2>nul
powershell -Command "
`$conf = @'
worker_processes auto;
error_log C:/oxnginx/wwwlogs/error.log warn;
events { worker_connections 1024; }
http {
    include mime.types;
    default_type application/octet-stream;
    access_log C:/oxnginx/wwwlogs/access.log;
    sendfile on;
    keepalive_timeout 65;
    include C:/oxnginx/server/nginx/conf/sites-enabled/*.conf;
}
'@
Set-Content '%INST%\server\nginx\conf\nginx.conf' `$conf
"

echo   [5/5] 注册服务...
powershell -Command "if(-not(Test-Path '%INST%\tools\nssm.exe')){Invoke-WebRequest -Uri 'https://nssm.cc/release/nssm-2.24.zip' -OutFile '%TEMP%\nssm.zip' -UseBasicParsing;Expand-Archive '%TEMP%\nssm.zip' '%TEMP%\nssm' -Force;Copy-Item '%TEMP%\nssm\nssm-2.24\win64\nssm.exe' '%INST%\tools\nssm.exe';Remove-Item -Recurse -Force '%TEMP%\nssm*'}"
"%INST%\tools\nssm.exe" stop OxNginx >nul 2>&1
"%INST%\tools\nssm.exe" remove OxNginx confirm >nul 2>&1
"%INST%\tools\nssm.exe" install OxNginx "%INST%\server\panel\ox-nginx.exe"
"%INST%\tools\nssm.exe" set OxNginx AppDirectory "%INST%\server\panel"
"%INST%\tools\nssm.exe" set OxNginx DisplayName "OxNginx"
"%INST%\tools\nssm.exe" set OxNginx Start SERVICE_AUTO_START
"%INST%\tools\nssm.exe" set OxNginx AppEnvironmentExtra "RUST_LOG=info" "CONFIG_PATH=%INST%\server\panel\configs\config.toml"
"%INST%\tools\nssm.exe" set OxNginx AppStdout "%INST%\wwwlogs\panel.log"
"%INST%\tools\nssm.exe" set OxNginx AppStderr "%INST%\wwwlogs\panel.log"
"%INST%\tools\nssm.exe" set OxNginx AppRotateFiles 1
"%INST%\tools\nssm.exe" set OxNginx AppRotateBytes 10485760
"%INST%\tools\nssm.exe" start OxNginx

echo.
echo   ========================================
echo   安装完成！
echo   ========================================
echo.
for /f "tokens=2 delims=:" %%a in ('ipconfig ^| findstr /i "IPv4" ^| findstr /v "127.0.0.1"') do set IP=%%a
echo   访问地址: http://%IP: =%:9000
echo   安装目录: %INST%
echo.
pause
"@
$Setup | Out-File -FilePath "$PkgDir\setup.cmd" -Encoding ASCII

# ============ 打包 ============
Write-Info "打包 zip..."
Compress-Archive -Path $PkgDir -DestinationPath $ZipFile
Remove-Item -Recurse -Force $PkgDir

$Size = [math]::Round((Get-Item $ZipFile).Length / 1MB, 2)
Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "  打包完成！" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host ""
Write-Host "  $PkgName.zip  ${Size}MB" -ForegroundColor Cyan
Write-Host ""
Write-Host "  解压后右键 setup.cmd，以管理员身份运行" -ForegroundColor Yellow
Write-Host ""
