# OxNginx 便携一键清除脚本
# 使用方法: .\scripts\clean.ps1 [-Force]
# 作用: 删除运行时生成的文件/文件夹，恢复到干净状态
#   backend/backup/  backend/configs/config.toml  backend/datas/
#   backend/server/  backend/ssl/  backend/wwwlogs/  backend/wwwroot/
# -Force: 跳过确认直接删除（用于无人值守/CI）

param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# 定位仓库根目录（脚本位于 scripts/ 下），保证任意 cwd 均可运行
$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BackendDir = Join-Path $RootDir "backend"

function Write-Info { param($msg) Write-Host "[✓] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[!] $msg" -ForegroundColor Yellow }

# 待清除项（相对 backend/），含目录与单个文件
$Targets = @(
    "backup",
    "configs\config.toml",
    "datas",
    "server",
    "ssl",
    "wwwlogs",
    "wwwroot"
)

# 展开为绝对路径，只保留实际存在的项
$Paths = @()
foreach ($t in $Targets) {
    $p = Join-Path $BackendDir $t
    if (Test-Path $p) { $Paths += $p }
}

Write-Host ""
Write-Host "OxNginx 一键清除脚本" -ForegroundColor Cyan
Write-Host ""

if ($Paths.Count -eq 0) {
    Write-Info "没有需要清除的内容，已是干净状态"
    return
}

Write-Warn "即将删除以下内容："
foreach ($p in $Paths) { Write-Host "    $p" }
Write-Host ""

# datas/(数据库) 与 ssl/(证书) 删除不可逆，默认需确认；-Force 跳过
if (-not $Force) {
    $ans = Read-Host "确认删除？(y/N)"
    if ($ans -notmatch "^[yY]") {
        Write-Warn "已取消"
        return
    }
}

foreach ($p in $Paths) {
    Remove-Item -Recurse -Force $p
    Write-Info "已删除 $p"
}

Write-Host ""
Write-Info "清除完成"
