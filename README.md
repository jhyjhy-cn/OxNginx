# OxNginx Manager

一个专注于轻量、高性能、低内存占用的 Nginx 可视化管理面板。

## 特性

- 🚀 极致轻量：常驻内存 < 20MB
- 🎯 单一二进制文件，无外部依赖
- 📊 系统监控 Dashboard
- 🌐 站点管理（CRUD）
- 🔒 SSL 证书管理（acme.sh）
- 🔄 Nginx 配置检测与重载
- 📝 日志查看
- 💾 配置备份与回滚

## 技术栈

- **后端**: Rust + Axum + Tokio + SQLite
- **前端**: Vue3 + TypeScript + Vite + Element Plus
- **认证**: JWT + Argon2id

## 开发环境

### 后端

```bash
cd backend
cargo run
```

### 前端

```bash
cd frontend
npm install
npm run dev
```

## 构建

```bash
# 构建前端
cd frontend
npm run build

# 构建后端
cd backend
cargo build --release
```

## 配置

编辑 `config.toml`：

```toml
[server]
port = 9000
host = "0.0.0.0"

[database]
path = "./data.db"

[nginx]
bin = "/usr/sbin/nginx"
config = "/etc/nginx/nginx.conf"
sites_enabled = "/etc/nginx/sites-enabled"

[acme]
bin = "/root/.acme.sh/acme.sh"

[auth]
jwt_secret = "change-this-to-a-random-secret-key"
jwt_expires_hours = 24
```

## API

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/login` | POST | 用户登录 |
| `/api/setup` | POST | 初始化管理员 |
| `/api/dashboard` | GET | Dashboard数据 |
| `/api/sites` | GET/POST | 站点列表/创建 |
| `/api/sites/:id` | GET/PUT/DELETE | 站点操作 |
| `/api/certificates` | GET | 证书列表 |
| `/api/certificate/apply` | POST | 申请证书 |
| `/api/nginx/test` | POST | 测试配置 |
| `/api/nginx/reload` | POST | 重载Nginx |
| `/api/log/access` | GET | Access日志 |
| `/api/log/error` | GET | Error日志 |

sqlite MCP安装 命令: uv tool install D:\jhy\Work\jhyjhy.cn\MCP\sqlite-mcp

## 许可证

MIT License
