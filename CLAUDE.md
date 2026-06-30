# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

OxNginx 是一个轻量级 Nginx 可视化管理面板，使用 Rust + Vue3 构建，目标内存占用 <20MB。

## 开发命令

### 后端
```bash
cd backend
cargo run              # 开发运行
cargo build --release  # 生产构建
cargo test             # 运行测试
```

### 前端
```bash
cd frontend
pnpm install           # 安装依赖
pnpm run dev           # 开发服务器
pnpm run build         # 生产构建
```

## 技术栈

- **后端**: Rust + Axum + Tokio + SQLite + sqlx
- **前端**: Vue3 + TypeScript + Vite + Pinia + Element Plus + Monaco Editor
- **认证**: JWT + Argon2id

## 架构设计

### 后端 Clean Architecture
```
API Layer (api/)      → 接收请求，返回统一 JSON { code, message, data }
       ↓
Service Layer (service/)  → 业务逻辑
       ↓
Repository/Database    → 数据持久化
```

### 核心原则
1. **按需读取**: 不做文件监听、日志索引、配置缓存。所有数据点击时读取。
2. **配置安全**: 所有 nginx 配置变更必须经过: 生成配置 → 备份 → `nginx -t` → 成功则 reload → 失败则回滚
3. **模板生成**: 配置文件使用模板生成，禁止字符串拼接
4. **异步执行**: 耗时任务（证书申请、备份等）使用 Tokio 异步执行

### 目录结构
```
backend/src/
├── api/           # API 处理器 (auth_api, site_api, nginx_api, log_api, backup_api)
├── service/       # 业务逻辑 (site_service, cert_service, backup_service, dashboard_service)
├── middleware/     # 认证中间件
├── nginx/         # Nginx 操作 (test, reload)
├── ssl/           # ACME/SSL 操作
├── backup/        # 备份管理
├── config/        # TOML 配置加载
├── database/      # SQLite + sqlx
├── dto/          # 请求/响应数据结构
├── model/         # 数据库模型
├── auth/          # JWT + Argon2id
└── main.rs        # 路由组装

frontend/src/
├── views/         # 页面组件 (Dashboard, Sites, SSL, Logs, Settings 等)
├── stores/        # Pinia 状态管理
├── router/        # Vue Router
└── api/           # Axios 封装
```

### API 路由
- 公开: `POST /api/login`, `POST /api/setup`
- 认证: `/api/dashboard`, `/api/sites`, `/api/certificates`, `/api/nginx/*`, `/api/log/*`, `/api/backups/*`

## 配置

`config.toml` 包含:
- `[server]` - HTTP 服务端口/host
- `[database]` - SQLite 路径
- `[nginx]` - nginx 二进制路径、配置文件路径、sites-enabled 目录
- `[acme]` - acme.sh 路径
- `[auth]` - JWT secret 和过期时间
