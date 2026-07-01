#!/bin/bash
# OxNginx Linux 部署脚本
# 使用方法: sudo bash deploy.sh

set -e

# ============ 配置区域 ============
APP_NAME="ox-nginx"
INSTALL_DIR="/opt/ox-nginx"
CONFIG_DIR="/opt/ox-nginx/configs"
DATA_DIR="/opt/ox-nginx/datas"
LOG_DIR="/opt/ox-nginx/logs"
SERVICE_USER="ox-nginx"

# ============ 颜色输出 ============
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; exit 1; }

# ============ 检查 root 权限 ============
if [ "$EUID" -ne 0 ]; then
    error "请使用 root 权限运行此脚本: sudo bash deploy.sh"
fi

# ============ 安装依赖 ============
info "安装系统依赖..."
# 不再需要 nginx，Rust 后端直接托管前端

# ============ 创建用户和目录 ============
info "创建系统用户和目录..."

# 创建用户（如果不存在）
if ! id "$SERVICE_USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$INSTALL_DIR" "$SERVICE_USER" 2>/dev/null || true
fi

# 创建目录结构
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$DATA_DIR"
mkdir -p "$LOG_DIR"

# ============ 复制文件 ============
info "部署应用文件..."

# 获取脚本所在目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# 查找后端二进制文件
BINARY_PATH=""
if [ -f "$PROJECT_DIR/bin/$APP_NAME" ]; then
    BINARY_PATH="$PROJECT_DIR/bin/$APP_NAME"
elif [ -f "$PROJECT_DIR/backend/target/x86_64-unknown-linux-gnu/release/$APP_NAME" ]; then
    BINARY_PATH="$PROJECT_DIR/backend/target/x86_64-unknown-linux-gnu/release/$APP_NAME"
elif [ -f "$PROJECT_DIR/backend/target/release/$APP_NAME" ]; then
    BINARY_PATH="$PROJECT_DIR/backend/target/release/$APP_NAME"
else
    error "后端二进制文件不存在，请检查 tar 包完整性"
fi

# 查找前端静态文件
STATIC_DIR=""
if [ -d "$PROJECT_DIR/static" ] && [ -f "$PROJECT_DIR/static/index.html" ]; then
    STATIC_DIR="$PROJECT_DIR/static"
elif [ -d "$PROJECT_DIR/backend/static" ]; then
    STATIC_DIR="$PROJECT_DIR/backend/static"
else
    error "前端静态文件不存在，请检查 tar 包完整性"
fi

# 复制文件
cp "$BINARY_PATH" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$APP_NAME"
cp -r "$STATIC_DIR" "$INSTALL_DIR/static"

# ============ 创建配置文件 ============
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    info "生成配置文件..."
    JWT_SECRET=$(openssl rand -base64 32)
    cat > "$CONFIG_DIR/config.toml" << EOF
[server]
port = 9000
host = "0.0.0.0"

[database]
path = "$DATA_DIR/data.db"

[nginx]
bin = "/usr/sbin/nginx"
config = "/etc/nginx/nginx.conf"
sites_enabled = "/etc/nginx/conf.d"

[acme]
bin = "/root/.acme.sh/acme.sh"

[auth]
jwt_secret = "$JWT_SECRET"
jwt_expires_hours = 24
EOF
    info "配置文件已生成: $CONFIG_DIR/config.toml"
else
    warn "配置文件已存在，跳过生成"
fi

# ============ 创建 systemd 服务 ============
info "创建系统服务..."

cat > /etc/systemd/system/$APP_NAME.service << EOF
[Unit]
Description=OxNginx - Nginx 可视化管理面板
After=network.target nginx.service
Wants=nginx.service

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/$APP_NAME
Environment=RUST_LOG=info
Environment=CONFIG_PATH=$CONFIG_DIR/config.toml
Restart=always
RestartSec=5
StandardOutput=append:$LOG_DIR/access.log
StandardError=append:$LOG_DIR/error.log

[Install]
WantedBy=multi-user.target
EOF

# ============ 安装 on 命令 ============
info "安装管理命令..."

cat > /usr/local/bin/on << 'ONSCRIPT'
#!/bin/bash
# OxNginx 管理菜单

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

show_menu() {
    clear
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║      OxNginx 管理面板 v1.0.0        ║${NC}"
    echo -e "${CYAN}╠══════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}1${NC} = 启动服务                        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}2${NC} = 停止服务                        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}3${NC} = 重启服务                        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}4${NC} = 查看状态                        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}5${NC} = 查看日志                        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}6${NC} = 面板信息                        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}7${NC} = 重置密码                        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  ${GREEN}0${NC} = 退出                            ${CYAN}║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
    echo ""
}

start_service() {
    echo -e "${GREEN}正在启动服务...${NC}"
    systemctl start ox-nginx
    echo -e "${GREEN}✅ 服务已启动${NC}"
    echo ""
    IP=$(hostname -I | awk '{print $1}')
    echo -e "🌐 访问地址: ${CYAN}http://${IP}:9000${NC}"
    echo ""
    read -p "按 Enter 返回菜单..."
}

stop_service() {
    echo -e "${YELLOW}正在停止服务...${NC}"
    systemctl stop ox-nginx
    echo -e "${GREEN}✅ 服务已停止${NC}"
    echo ""
    read -p "按 Enter 返回菜单..."
}

restart_service() {
    echo -e "${YELLOW}正在重启服务...${NC}"
    systemctl restart ox-nginx
    echo -e "${GREEN}✅ 服务已重启${NC}"
    echo ""
    read -p "按 Enter 返回菜单..."
}

show_status() {
    echo -e "${CYAN}========== OxNginx 状态 ==========${NC}"
    systemctl status ox-nginx --no-pager
    echo ""
    read -p "按 Enter 返回菜单..."
}

show_logs() {
    echo -e "${CYAN}========== 实时日志 (Ctrl+C 退出) ==========${NC}"
    journalctl -u ox-nginx -f --no-pager
}

show_info() {
    IP=$(hostname -I | awk '{print $1}')
    CONFIG_FILE="/opt/ox-nginx/configs/config.toml"
    JWT_SECRET="未知"
    if [ -f "$CONFIG_FILE" ]; then
        JWT_SECRET=$(grep 'jwt_secret' "$CONFIG_FILE" | cut -d'"' -f2)
    fi

    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║         OxNginx 面板信息             ║${NC}"
    echo -e "${CYAN}╠══════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC} 版本:     v1.0.0                     ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} 访问地址: http://${IP}:9000          ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} 配置目录: /opt/ox-nginx/configs      ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} 数据目录: /opt/ox-nginx/datas        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} 日志目录: /opt/ox-nginx/logs         ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} 前端目录: /opt/ox-nginx/static       ${CYAN}║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
    echo ""
    read -p "按 Enter 返回菜单..."
}

reset_password() {
    echo ""
    echo -e "${YELLOW}⚠️  重置密码将清除所有用户数据！${NC}"
    read -p "确定要重置密码吗？(y/N): " confirm
    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
        # 删除数据库，重启服务后会自动重新初始化
        rm -f /opt/ox-nginx/datas/data.db
        systemctl restart ox-nginx
        echo -e "${GREEN}✅ 密码已重置，请重新访问面板进行初始化设置${NC}"
    else
        echo -e "${YELLOW}已取消${NC}"
    fi
    echo ""
    read -p "按 Enter 返回菜单..."
}

# 主循环
while true; do
    show_menu
    read -p "请输入数字 [0-7]: " choice

    case $choice in
        1) start_service ;;
        2) stop_service ;;
        3) restart_service ;;
        4) show_status ;;
        5) show_logs ;;
        6) show_info ;;
        7) reset_password ;;
        0) echo -e "${GREEN}再见！${NC}"; exit 0 ;;
        *) echo -e "${RED}无效选项，请重新输入${NC}"; sleep 1 ;;
    esac
done
ONSCRIPT

chmod +x /usr/local/bin/on

# ============ 设置权限 ============
info "设置文件权限..."

chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"

# ============ 启动服务 ============
info "启动服务..."

systemctl daemon-reload
systemctl enable $APP_NAME
systemctl restart $APP_NAME

# ============ 完成 ============
IP=$(hostname -I | awk '{print $1}')
echo ""
echo "=========================================="
echo -e "${GREEN}✅ OxNginx 部署成功！${NC}"
echo "=========================================="
echo ""
echo -e "🌐 访问地址: ${CYAN}http://${IP}:9000${NC}"
echo ""
echo -e "📋 管理命令: 输入 ${GREEN}on${NC} 呼出管理菜单"
echo ""
echo "=========================================="
