# OxNginx Linux 部署指南

## 方案一：自动化部署（推荐）

### 1. 在 Windows 上交叉编译

```bash
# 安装 Linux 编译目标
rustup target add x86_64-unknown-linux-gnu

# 安装交叉编译工具（需要先安装 scoop）
scoop install zig

# 安装 cargo-zigbuild
cargo install cargo-zigbuild

# 编译后端
cd backend
cargo zigbuild --release --target x86_64-unknown-linux-gnu

# 构建前端
cd ../frontend
pnpm install
pnpm run build
```

### 2. 上传到服务器

```bash
# 打包部署文件
tar -czf ox-nginx.tar.gz \
    backend/target/x86_64-unknown-linux-gnu/release/ox-nginx \
    frontend/dist \
    deploy.sh

# 上传到服务器
scp ox-nginx.tar.gz user@your-server:/opt/

# 在服务器上解压
ssh user@your-server
cd /opt
tar -xzf ox-nginx.tar.gz
chmod +x deploy.sh
sudo ./deploy.sh
```

---

## 方案二：在 Linux 上直接编译

### 1. 安装 Rust 和 Node.js

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装 Node.js (推荐 v18+)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# 安装 pnpm
npm install -g pnpm
```

### 2. 克隆并编译项目

```bash
# 克隆项目
git clone https://github.com/jhyjhy-cn/OxNginx.git
cd OxNginx

# 编译后端
cd backend
cargo build --release
cd ..

# 构建前端
cd frontend
pnpm install
pnpm run build
cd ..

# 运行部署脚本
chmod +x deploy.sh
sudo ./deploy.sh
```

---

## 方案三：Docker 部署（最简单）

### Dockerfile

```dockerfile
# 构建阶段
FROM rust:1.75-slim as builder

WORKDIR /app
COPY backend/ ./backend/
COPY frontend/ ./frontend/

# 安装 Node.js
RUN apt-get update && apt-get install -y curl
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
RUN apt-get install -y nodejs
RUN npm install -g pnpm

# 构建前端
WORKDIR /app/frontend
RUN pnpm install && pnpm run build

# 构建后端
WORKDIR /app/backend
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    nginx \
    certbot \
    python3-certbot-nginx \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/ox-nginx

# 复制构建产物
COPY --from=builder /app/backend/target/release/ox-nginx .
COPY --from=builder /app/frontend/dist ./static

# 复制配置
COPY deploy/nginx.conf /etc/nginx/conf.d/ox-nginx.conf
COPY deploy/config.toml /etc/ox-nginx/config.toml

# 创建目录
RUN mkdir -p /var/lib/ox-nginx /var/log/ox-nginx

EXPOSE 80 9000

# 启动脚本
COPY deploy/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  ox-nginx:
    build: .
    ports:
      - "80:80"
    volumes:
      - ox-nginx-data:/var/lib/ox-nginx
      - ox-nginx-config:/etc/ox-nginx
      - nginx-config:/etc/nginx
      - /var/log/nginx:/var/log/nginx
    restart: unless-stopped
    privileged: true  # 需要管理 nginx

volumes:
  ox-nginx-data:
  ox-nginx-config:
  nginx-config:
```

### 运行

```bash
# 构建并启动
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止
docker-compose down
```

---

## 部署后配置

### 1. 修改配置文件

```bash
sudo nano /etc/ox-nginx/config.toml
```

重要配置项：
- `auth.jwt_secret` - 修改为随机密钥
- `nginx.bin` - nginx 可执行文件路径
- `nginx.config` - nginx 配置文件路径
- `nginx.sites_enabled` - 站点配置目录

### 2. 生成 JWT 密钥

```bash
# 生成随机密钥
openssl rand -base64 32

# 将生成的密钥填入 config.toml 的 jwt_secret
```

### 3. 配置 HTTPS（可选）

```bash
# 使用 certbot 申请证书
sudo certbot --nginx -d your-domain.com

# 自动续期测试
sudo certbot renew --dry-run
```

### 4. 配置防火墙

```bash
# Ubuntu/Debian
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# CentOS/RHEL
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --reload
```

---

## 服务管理

```bash
# 启动服务
sudo systemctl start ox-nginx

# 停止服务
sudo systemctl stop ox-nginx

# 重启服务
sudo systemctl restart ox-nginx

# 查看状态
sudo systemctl status ox-nginx

# 查看日志
sudo tail -f /var/log/ox-nginx/access.log
sudo tail -f /var/log/ox-nginx/error.log

# 开机自启
sudo systemctl enable ox-nginx

# 取消自启
sudo systemctl disable ox-nginx
```

---

## 目录结构

```
/opt/ox-nginx/          # 应用目录
├── ox-nginx            # 后端二进制
└── static/             # 前端静态文件
    ├── index.html
    └── assets/

/etc/ox-nginx/          # 配置目录
└── config.toml         # 主配置文件

/var/lib/ox-nginx/      # 数据目录
└── data.db             # SQLite 数据库

/var/log/ox-nginx/      # 日志目录
├── access.log
└── error.log

/etc/nginx/conf.d/      # Nginx 配置
└── ox-nginx.conf       # 反向代理配置
```

---

## 常见问题

### Q: 权限问题
```bash
# 确保 nginx 用户有权限访问
sudo chown -R ox-nginx:ox-nginx /var/lib/ox-nginx
sudo chown -R ox-nginx:ox-nginx /var/log/ox-nginx
```

### Q: 端口被占用
```bash
# 查看端口占用
sudo lsof -i :9000
sudo lsof -i :80

# 修改配置文件中的端口
sudo nano /etc/ox-nginx/config.toml
```

### Q: nginx 操作失败
```bash
# 确保用户在 nginx 组中
sudo usermod -aG nginx ox-nginx

# 或者使用 sudoers 配置
sudo visudo
# 添加: ox-nginx ALL=(ALL) NOPASSWD: /usr/sbin/nginx
```

### Q: 数据库锁定
```bash
# 停止服务后操作数据库
sudo systemctl stop ox-nginx
# 操作完成后重启
sudo systemctl start ox-nginx
```
