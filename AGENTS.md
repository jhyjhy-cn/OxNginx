# OxNginx — Agent 快速参考

## 仓库结构

```
backend/          # Rust 服务端 (Axum + SQLite + sqlx)
backend-gui/      # Tauri 包装 (Windows 托盘应用 + 自动更新)
frontend/         # Vue3 SPA (Vite + Element Plus + Tailwind CSS 4)
libs/nginx/       # 捆绑的 nginx 二进制 (Windows zip, Linux 源码)
scripts/          # build-win.ps1, deploy.sh (Linux systemd)
VERSION           # 版本号的唯一来源
```

## 开发者命令

```bash
# 后端
cd backend
cargo run                    # 开发服务器 (单线程 Tokio runtime)
cargo build --release        # LTO + opt-level z + strip
cargo test

# 前端 (需 pnpm, node >= 20.19.0)
cd frontend
pnpm install
pnpm run dev                 # Vite 在 :3000, 代理 /api -> :9000
pnpm run build               # vue-tsc 类型检查 + vite build
pnpm run format              # oxfmt
pnpm run lint                # oxlint
pnpm run lint:fix            # oxlint --fix

# 完整构建 (Windows)
.\scripts\build-win.ps1

# 完整构建 (Linux 交叉编译)
cargo zigbuild --target x86_64-unknown-linux-gnu --release   # 需要 cargo-zigbuild
sudo bash scripts/deploy.sh                                   # 部署到 /opt/oxnginx + systemd
```

## 后端架构

- **运行时**: Tokio `new_current_thread()` (单线程), SQLite `max_connections=1`, 目标内存 <20MB
- **入口**: `backend/src/main.rs` — 启动顺序: setup → config → log → db → RSA 密钥 → 审计 worker → WS 推送任务 → router → 绑定端口
- **Clean Architecture 模块**: 每个功能在 `modules/<feature>/` 下, 含子目录:
  - `controller/` — Axum 处理器 (State<AppState> + Json/Path/Extension, 返回 `Json<ApiResponse<T>>`)
  - `service/` — 业务逻辑 (接收 `&AppState`)
  - `dao/` — 通过 `sqlx::query_as::<_, Entity>()` 写原始 SQL (无 repository 层)
  - `entity/` — 带 `sqlx::FromRow` derive 的结构体
- **公共模块**: `modules/common/` — config, database, auth (RSA+JWT), middleware, DTO, 审计日志, nginx 辅助函数
- **AppState** (`app/state.rs`): 持有 `db: Database`, `config: Arc<Mutex<AppConfig>>`, `sys: System`, `pid`, `dashboard_tx: broadcast`, `rsa_private_key`, `rsa_public_key_b64`
- **路由** (`app/router.rs`): 三层 — `public_routes` → `protected_routes` → `admin_routes`, 合并 CORS + 静态文件 fallback
- **配置**: `config.toml` 从 `CONFIG_PATH` 环境变量或 `{exe_dir}/configs/config.toml` 加载。首次运行通过 `startup/setup.rs` 自动生成 (捆绑 nginx.zip, 生成随机 JWT secret)
- **安全**: 密码客户端侧通过 JSEncrypt + RSA 公钥加密 (`/api/auth/public-key`)。登录体: `{ username, encrypted_password }`。JWT 用 RSA 私钥签名
- **RBAC**: 用户、角色、菜单、部门、岗位、字典 — 都在 `modules/sys/` 下。管理员路由在 `/api/rbac/*`
- **审计日志**: 处理器上的 `#[audit_log(module, action, capture = req)]` 宏。异步 channel 批量写入

## 前端架构

- **动态路由**: 从 RBAC 菜单树注册路由 (`router/index.ts`)。`import.meta.glob('../views/**/*.vue')` 懒加载组件
- **认证 store** (`stores/auth.ts`): token + username 存 localStorage, RBAC roles/perms/menus 从 `/api/rbac/me` 获取。`isSuperAdmin = username === 'admin' || roles.includes('super_admin')`
- **设置 store** (`stores/settings.ts`): 通过 `pinia-plugin-persistedstate` 持久化。控制主题、布局模式 (`sidebar-tree` / `top-tree`)、语言、暗色模式
- **国际化**: `vue-i18n` + 数据库驱动翻译 (`/api/rbac/i18n`)。登录后加载
- **API 客户端** (`api/index.ts`): Axios 拦截器添加 Bearer token, 401 自动登出, 403 跳转
- **布局**: `layouts/MainLayout.vue`, `SidebarDoubleLayout.vue`, `SidebarTreeLayout.vue`, `TopTreeLayout.vue`, `ThemeDrawer.vue`
- **关键组件**: `OnIcon`, `OnDialog`, `OnForm`, `OnTable`, `OnPagination`, `HasPermission` — CRUD UX 封装
- **Monaco 编辑器**: ConfigEditor 使用, 环境设置在 `utils/monaco-env`。预构建在 Vite optimizeDeps
- **xterm**: WebSocket 终端视图 (`/api/terminal/ws`)

## 关键约定

1. **不要加 repository 层** — DAO 文件直接在 `modules/<feature>/dao/` 下, 由 service 调用
2. **SQL 通过 `sqlx::query_as::<_, Model>`** — 无 ORM, DAO 文件中写原始 SQL
3. **Nginx 配置安全**: 生成 → 备份 → `nginx -t` → 成功则 reload, 失败则回滚。基于模板, 禁止字符串拼接
4. **统一 API 响应**: `ApiResponse<T>` → `{ code: 0|-1, message, data }`。成功用 `ApiResponse::success(data)`, 失败用 `ApiResponse::<()>::error(msg)`
5. **前端 `build.outDir`**: `../backend/target/debug/static` — 开发构建输出到这里热重载。生产构建输出到 `backend/static/`
6. **版本号唯一来源**: 根目录 `VERSION` 文件。`scripts/build-win.ps1` 读取它。`Cargo.toml` 和 `tauri.conf.json` 需手动同步
7. **静态文件**: 从 `{exe_dir}/static/` 提供 (SPA fallback 到 index.html)。前端 `build` 输出到此

## 新增模块步骤

1. 创建 `modules/<name>/` 含 `controller/`, `service/`, `dao/`, `entity/` 子目录 (及 `mod.rs`)
2. 在 `controller/` 添加处理器 — 接收 `State<AppState>` + `Json<Req>`, 调用 `service::fn()`, 返回 `Json<ApiResponse<T>>`
3. 在 `app/router.rs` 注册路由 (public / protected / admin 层)
4. 前端: 在 `frontend/src/views/<name>/` 添加视图, `frontend/src/stores/` 添加 store, 菜单配置中加路由项
5. 如有初始数据, 在 `modules/common/database/seed.rs` 中播种

## 已知陷阱

- `vite.config.ts` 用 `fileURLToPath` 推导 `__dirname` (Vite 8 仅 ESM)
- `element-plus` 2.13 需要在 vite 配置中设 `scss api: 'modern-compiler'`
- Monaco editor 顶层入口必须在 `optimizeDeps.include` 中, 否则开发时卡顿
- Tauri GUI (`backend-gui/`) 包装同一个后端二进制, 增加托盘 + 自动更新 (GitHub Releases)
- Windows NSIS 安装包: 双语 (SimpChinese + English), 每台机器安装模式
- Linux 部署脚本创建 `on` 命令在 `/usr/local/bin/on`, 用于服务管理菜单
