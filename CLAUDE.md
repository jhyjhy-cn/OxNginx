# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

OxNginx 是一个轻量级 Nginx 可视化管理面板，使用 Rust + Vue3 构建，目标内存占用 <20MB。

## 开发命令

### 后端
```bash
cd backend
cargo run              # 开发运行（单线程 Tokio runtime）
cargo run -- --console # Windows release 窗口程序需要控制台时使用
cargo build --release  # 生产构建（LTO + opt-level z + strip）
cargo test             # 运行测试
cargo test <name>      # 运行单个测试
```

### 前端
```bash
cd frontend
pnpm install           # 安装依赖（项目使用 pnpm，非 npm；Node >= 20.19.0）
pnpm run dev           # 开发服务器（Vite :3000，代理 /api -> :9000）
pnpm run build         # 生产构建（vue-tsc 类型检查 + vite build）
pnpm run preview       # 预览构建结果
pnpm run lint          # oxlint 检查 src/
pnpm run lint:fix      # oxlint 自动修复
pnpm run format        # oxfmt 格式化 src/**/*.{vue,ts,tsx,js,jsx,json,css,scss}
```

### 桌面壳（Tauri）
```bash
cd backend-gui
cargo run              # 运行 Tauri2 GUI 管理器（托盘 + 自动更新）
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

## 仓库结构

```
backend/        # Rust 服务端
backend-gui/    # Tauri 包装（托盘 + GitHub Releases 自动更新）
frontend/       # Vue3 SPA
libs/nginx/     # 捆绑的 nginx 二进制（Windows zip，Linux 源码）
scripts/        # build-win.ps1, deploy.sh, build-linux.ps1
VERSION         # 版本号唯一来源（需手动同步 Cargo.toml 和 tauri.conf.json）
```

## 后端架构（SpringBoot 风格 modules）

```
backend/src/
├── main.rs        # 启动入口；setup → config → log → db → RSA → 审计 worker → WS 推送 → router → 绑定端口
├── app/           # 应用层（router.rs / state.rs）
├── startup/       # 启动初始化（setup, logging）
└── modules/       # 业务模块，每个含 controller/ + service/ + dao/ + entity/ 四件套
    ├── auth/      # 认证
    ├── backup/    # 备份管理
    ├── common/    # 公共（config, database, auth, middleware, DTO, audit, nginx 辅助）
    ├── dashboard/ # Dashboard 数据与 WebSocket 推送
    ├── file/      # 文件管理
    ├── log/       # 日志查看
    ├── nginx/     # Nginx 操作（test, reload）
    ├── settings/  # 系统设置
    ├── site/      # 站点管理
    ├── sys/       # 系统信息
    └── system/    # 系统配置
```

### 关键架构约束

- **单线程运行时**: Tokio `new_current_thread()`，SQLite `max_connections=1`，目标 <20MB 内存。
- **无 ORM/无 Repository 层**: Service 层直接写 SQL 字符串，通过 `sqlx::query_as::<_, Model>(sql)` 执行；DAO 文件直接放在 `modules/<feature>/dao/`，由 service 调用。
- **统一响应**: `ApiResponse<T>` → `{ code: 0|-1, message, data }`；成功 `ApiResponse::success(data)`，失败 `ApiResponse::<()>::error(msg)`。
- **按需读取**: 不做文件监听、日志索引、配置缓存，所有数据点击时读取。
- **Nginx 配置安全**: 生成 → 备份 → `nginx -t` → 成功则 reload，失败则回滚。模板生成，禁止字符串拼接。
- **共享状态**: `AppState` (`app/state.rs`) 持有 `Database`、`Arc<Mutex<AppConfig>>`、`System`、`pid`、`dashboard_tx: broadcast`、`rsa_private_key`、`rsa_public_key_b64`。
- **路由分层** (`app/router.rs`): `public_routes` → `protected_routes` → `admin_routes`，合并 CORS + 静态文件 SPA fallback。
- **首次启动自引导**: `startup/setup.rs` 检测 bundled nginx.zip，自动生成 config.toml（含随机 JWT secret）、解压 nginx、注册 Windows NSSM 服务。
- **运行时配置路径**: `CONFIG_PATH` 环境变量，回退到 `{exe_dir}/configs/config.toml`。
- **静态文件**: 从 `{exe_dir}/static/` 提供，SPA fallback 到 index.html；前端 `build` 输出到此。
- **前端 build 输出**: 开发模式 `outDir = ../backend/target/debug/static`（热重载），生产模式 `backend/static/`。

### 认证流程

- 密码客户端侧通过 **JSEncrypt + RSA 公钥** 加密（`/api/auth/public-key`）。
- 登录体: `{ username, encrypted_password }`。
- JWT 用 RSA 私钥签名。

### RBAC

- 用户、角色、菜单、部门、岗位、字典都在 `modules/sys/` 下；管理员路由在 `/api/rbac/*`。
- 超级管理员判定：`username === 'admin' || roles.includes('super_admin')`。

### 审计日志

处理器上挂 `#[audit_log(module, action, capture = req)]` 宏，异步 channel 批量写库。

### API 处理器模式
```rust
// modules/xxx/controller/xxx_api.rs
pub async fn handler(State(state): State<AppState>, Json(req): Json<SomeRequest>) -> Json<Value> {
    match xxx_service::do_something(&state, req).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(&e.to_string()))),
    }
}
```

## 前端架构

```
frontend/src/
├── views/         # 页面组件；动态路由组件路径来自后端菜单 component 字段
├── stores/        # Pinia 状态管理（auth/settings 等用 pinia-plugin-persistedstate 持久化）
├── router/        # Vue Router；按后端菜单动态 addRoute，按 meta.permission 守卫
├── api/           # Axios 封装；拦截器加 Bearer，401 自动登出，403 跳转
├── components/    # 通用组件（OnIcon, OnDialog, OnForm, OnTable, OnPagination, HasPermission）
├── composables/   # 组合式函数
├── hooks/         # 自定义 hooks
├── config/menu.ts # 菜单图标兜底映射
└── i18n/          # 国际化 (zh-CN, en-US)
```

- **动态路由**: 登录后从 RBAC 菜单树注册路由。`router/index.ts` 用 `import.meta.glob('../views/**/*.vue')` 按菜单 `component` 字段懒加载。
- **认证 store** (`stores/auth.ts`): token + username 存 localStorage，roles/perms/menus 从 `/api/rbac/me` 获取。
- **设置 store** (`stores/settings.ts`): 持久化主题、布局模式（`sidebar-tree` / `top-tree`）、语言、暗色模式。
- **国际化**: `vue-i18n` + 数据库驱动翻译 (`/api/rbac/i18n`)，登录后加载。
- **Monaco 编辑器**: `utils/monaco-env` 配置，顶层入口必须在 `optimizeDeps.include`。
- **xterm**: WebSocket 终端视图 `/api/terminal/ws`。
- **布局**: `MainLayout.vue` / `SidebarDoubleLayout.vue` / `SidebarTreeLayout.vue` / `TopTreeLayout.vue` / `ThemeDrawer.vue`。

## API 路由
- 公开: `POST /api/login`, `POST /api/setup`
- 认证: `/api/dashboard`, `/api/sites`, `/api/certificates`, `/api/nginx/*`, `/api/log/*`, `/api/backups/*`

## 新增模块步骤

1. 后端建 `modules/<name>/` 含 `controller/`、`service/`、`dao/`、`entity/` 子目录（及 `mod.rs`）。
2. 在 `controller/` 加处理器 — 接 `State<AppState>` + `Json<Req>`，调 `service::fn()`，返回 `Json<ApiResponse<T>>`。
3. 在 `app/router.rs` 注册到对应层（public / protected / admin）。
4. 前端在 `frontend/src/views/<name>/` 加视图，`stores/` 加 store，菜单配置加路由项。
5. 初始数据在 `modules/common/database/seed.rs` 播种。

## 已知陷阱

- `vite.config.ts` 用 `fileURLToPath` 推导 `__dirname`（Vite 8 仅 ESM）。
- `element-plus` 2.13 需在 vite 配置设 `scss api: 'modern-compiler'`。
- Monaco editor 顶层入口必须在 `optimizeDeps.include`，否则开发时卡顿。
- Tauri GUI (`backend-gui/`) 包装同一个后端二进制，增加托盘 + 自动更新（GitHub Releases）。
- Windows NSIS 安装包双语（SimpChinese + English），每台机器安装模式。
- Linux 部署脚本创建 `on` 命令到 `/usr/local/bin/on`，用于服务管理菜单。
- 版本号唯一来源是根目录 `VERSION`，`scripts/build-win.ps1` 读取它；`Cargo.toml` 和 `tauri.conf.json` 需手动同步。

## 配置

`config.toml` 包含:
- `[server]` - HTTP 服务端口/host
- `[database]` - SQLite 路径、SQL 日志开关
- `[acme]` - acme.sh 路径
- `[auth]` - JWT secret 和过期时间
- `[log]` - 日志级别与轮转大小