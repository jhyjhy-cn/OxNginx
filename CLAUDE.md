# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

OxNginx 是一个轻量级 Nginx 可视化管理面板，使用 Rust + Vue3 构建，目标内存占用 <20MB。

## 开发命令

### 后端
```bash
cd backend
cargo run              # 开发运行
cargo run -- --console # Windows release 窗口程序需要控制台时使用
cargo build --release  # 生产构建（LTO + strip，用于发布）
cargo test             # 运行测试
cargo test <name>      # 运行单个测试
```

### 前端
```bash
cd frontend
pnpm install           # 安装依赖（项目使用 pnpm，非 npm；Node >= 20.19.0）
pnpm run dev           # 开发服务器
pnpm run build         # 生产构建（vue-tsc 类型检查 + vite build）
pnpm run preview       # 预览构建结果
pnpm run lint          # oxlint 检查 src/
pnpm run lint:fix      # oxlint 自动修复
pnpm run format        # oxfmt 格式化 src/**/*.{vue,ts,tsx,js,jsx,json,css,scss}
```

### 桌面壳（Tauri）
```bash
cd backend-gui
cargo run              # 运行 Tauri2 GUI 管理器
cargo build --release  # 构建 GUI
```

### 打包部署
```bash
./scripts/build-win.ps1      # Windows 构建（前端 + cargo-packager NSIS 安装包）
./scripts/build-win-gui.ps1  # Windows GUI 构建
./scripts/build-linux.ps1    # Windows 上交叉构建 Linux 产物
./scripts/deploy.sh          # Linux 部署到 /opt/oxnginx，systemd 服务 ox-nginx
# Linux 交叉编译: cd backend && cargo zigbuild --release --target x86_64-unknown-linux-gnu
```

## 技术栈

- **后端**: Rust + Axum + Tokio + SQLite + sqlx
- **前端**: Vue3 + TypeScript + Vite + Pinia + Element Plus + Tailwind CSS + Monaco Editor + xterm
- **桌面壳**: Tauri2（backend-gui）
- **认证**: JWT + RSA 登录密码加密 + Argon2id 密码哈希

## 架构设计

### 后端 Clean Architecture（SpringBoot 风格 modules）
```
backend/src/
├── main.rs        # 启动入口；自引导、加载配置、初始化日志/DB、启动 Axum
├── app/           # 应用层（router.rs / state.rs）
├── startup/       # 启动初始化（setup, logging）
└── modules/       # 业务模块
    ├── auth/      # 认证（controller / service / dao / entity）
    ├── backup/    # 备份管理
    ├── common/    # 公共模块（config, database, auth, audit）
    ├── dashboard/ # Dashboard 数据与 WebSocket 推送
    ├── file/      # 文件管理
    ├── log/       # 日志查看
    ├── nginx/     # Nginx 操作（test, reload）
    ├── settings/  # 系统设置
    ├── site/      # 站点管理
    ├── sys/       # 系统信息
    └── system/    # 系统配置
```

### 核心原则
1. **按需读取**: 不做文件监听、日志索引、配置缓存。所有数据点击时读取。
2. **配置安全**: 所有 nginx 配置变更必须经过：生成配置 → 备份 → `nginx -t` → 成功则 reload → 失败则回滚。
3. **模板生成**: 配置文件使用模板生成，禁止字符串拼接。
4. **异步执行**: 耗时任务（证书申请、备份等）使用 Tokio 异步执行。

### 关键架构约束
- **单线程运行时**: Tokio `new_current_thread()`，SQLite `max_connections=1`，目标 <20MB 内存。
- **无 ORM**: Service 层直接写 SQL 字符串通过 `sqlx::query_as::<_, Model>(sql)` 执行，不要引入 repository 层。
- **统一响应**: API 返回 `ApiResponse<T>` → `{ code: 0|-1, message, data }`；成功用 `ApiResponse::success(data)`，失败用 `ApiResponse::<()>::error(msg)`。
- **共享状态**: `AppState` 持有 `Database`、`Arc<Mutex<AppConfig>>`、`System`、Dashboard broadcast channel、RSA 密钥。
- **首次启动自引导**: `startup/setup.rs` 检测 bundled nginx.zip，自动生成 config.toml（含随机 JWT secret）、解压 nginx、注册 Windows NSSM 服务。
- **运行时配置路径**: `CONFIG_PATH` 环境变量，回退到 `{exe_dir}/configs/config.toml`。

### 启动流程
1. 首次运行检测 & 自引导（setup）。
2. 加载/生成 config.toml。
3. 初始化日志（tracing + 文件轮转）。
4. 初始化数据库。
5. 生成 RSA 密钥对（登录密码解密用，公钥给前端）。
6. 启动 Dashboard WebSocket 定时推送任务。
7. 启动操作日志异步 worker（channel 批量写库）。
8. 构建路由并监听端口。

### API 处理器模式
```rust
// modules/xxx/controller/xxx_api.rs — 接收 AppState + Json 请求体，调用 service，返回统一 JSON
pub async fn handler(State(state): State<AppState>, Json(req): Json<SomeRequest>) -> Json<Value> {
    match xxx_service::do_something(&state, req).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(&e.to_string()))),
    }
}
```

### 前端结构与路由
```
frontend/src/
├── views/         # 页面组件；动态路由组件路径来自后端菜单 component 字段
├── stores/        # Pinia 状态管理，包含认证/RBAC/设置等持久化状态
├── router/        # Vue Router；根据后端菜单动态 addRoute，并按 meta.permission 守卫
├── api/           # Axios 封装；请求拦截加 Bearer，401 自动登出
├── components/    # 通用组件
├── composables/   # 组合式函数
├── hooks/         # 自定义 hooks
├── config/menu.ts # 菜单图标兜底映射
└── i18n/          # 国际化 (zh-CN, en-US)
```

前端登录后拉取 RBAC 信息和菜单，`router/index.ts` 用 `import.meta.glob('../views/**/*.vue')` 按菜单的 `component` 字段注册动态路由。普通用户访问路由时检查 `meta.permission`，超级管理员跳过权限检查。

## API 路由
- 公开: `POST /api/login`, `POST /api/setup`
- 认证: `/api/dashboard`, `/api/sites`, `/api/certificates`, `/api/nginx/*`, `/api/log/*`, `/api/backups/*`

## 配置

`config.toml` 包含:
- `[server]` - HTTP 服务端口/host
- `[database]` - SQLite 路径、SQL 日志开关
- `[nginx]` - nginx 可执行文件路径、配置文件路径、sites-enabled 目录
- `[acme]` - acme.sh 路径
- `[auth]` - JWT secret 和过期时间
- `[log]` - 日志级别与轮转大小
