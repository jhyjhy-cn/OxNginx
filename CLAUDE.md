# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

OxNginx 是一个轻量级 Nginx 可视化管理面板，使用 Rust + Vue3 构建，目标内存占用 <20MB。

## 开发命令

### 后端
```bash
cd backend
cargo run              # 开发运行
cargo build --release  # 生产构建（LTO + strip，用于发布）
cargo test             # 运行测试
cargo test <name>      # 运行单个测试
```

### 前端
```bash
cd frontend
pnpm install           # 安装依赖（项目使用 pnpm，非 npm）
pnpm run dev           # 开发服务器
pnpm run build         # 生产构建（vue-tsc 类型检查 + vite build）
pnpm run preview       # 预览构建结果
```

### 打包部署
```bash
./scripts/build-win.ps1    # Windows 构建（前端 + cargo-packager NSIS 安装包）
./scripts/deploy.sh        # Linux 部署到 /opt/oxnginx，systemd 服务 ox-nginx
# Linux 交叉编译: cargo zigbuild --target x86_64-unknown-linux-gnu --release
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

### 关键架构约束
- **单线程运行时**: Tokio `new_current_thread()`，SQLite `max_connections=1`，目标 <20MB 内存
- **无 ORM**: Service 层直接写 SQL 字符串通过 `sqlx::query_as::<_, Model>(sql)` 执行，不要引入 repository 层
- **统一响应**: API 返回 `ApiResponse<T>` → `{ code: 0|-1, message, data }`，成功用 `ApiResponse::success(data)`，失败用 `ApiResponse::<()>::error(msg)`
- **首次启动自引导**: `startup/setup.rs` 检测 bundled nginx.zip，自动生成 config.toml（含随机 JWT secret）、解压 nginx、注册 Windows NSSM 服务
- **运行时配置路径**: `CONFIG_PATH` 环境变量，回退到 `{exe_dir}/configs/config.toml`

### API 处理器模式
```rust
// api/xxx_api.rs — 接收 AppState + Json 请求体，调用 service，返回统一 JSON
pub async fn handler(State(state): State<AppState>, Json(req): Json<SomeRequest>) -> Json<Value> {
    match xxx_service::do_something(&state, req).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(&e.to_string()))),
    }
}
```

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
├── stores/        # Pinia 状态管理 (auth/settings/files/tabs，后三者用 persistedstate)
├── router/        # Vue Router (history 模式，路由守卫检查 isAuthenticated)
├── api/           # 单文件 Axios 封装 (api/index.ts)，请求拦截加 Bearer，401 自动登出
├── config/menu.ts # 菜单定义：flatMenuItems + groupedMenuItems，标签用 i18n key
└── i18n/          # 国际化 (zh-CN, en-US)
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
