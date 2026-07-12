# OxNginx 优化待办清单

> 截止 2026-07-13，已落地的优化见 `git log`（15 个 commit）。
> 本清单按优先级排序，每条标注文件:行号证据、改法简述、风险等级。
> 已分两轮体检：第一轮抓 unwrap/lock/CI 等显性问题；第二轮抓架构债与静默失败。

---

## 🟢 P0 + P1 — 已完成（11 项）

| commit | 主题 |
|---|---|
| `95f230d` | parking_lot 替换 std Mutex + AppState 改 RwLock |
| `9cf2020` | SQL 路径 unwrap → `?`（token_dao + 4 service + user/dept parent_id） |
| `77c7165` | IO 路径 unwrap → `?`（site_backup_controller HeaderName 解析重构 + 6 处 expect/Result） |
| `6862e16` | ElementPlus 按需引入（删 `app.use(ElementPlus)` + 主 css） |
| ~~vue-router ^5.0.0~~ | **撤回**：实测是 posva 官方 vue-router 5.1.0 |
| `f0c1ca1` | `create_backup` 改 spawn_blocking |
| `718dd52` | version.ps1 全量同步 + 中文注释 + BOM |
| `93b5956` | EP dark css-vars 放到 main.css 之后 |
| `c3e9cf1` | seed_menu.rs 653→328 行，数据搬 menu.json |
| `cf21fde` | site_controller.rs 589→353，拆 batch/ssl 子文件 |

---

## 🔴 第二轮新发现（HIGH 安全/合规）

### 1. 文件 API 无路径白名单（认证用户即可读/写/删主机任意文件）
- **位置**：`backend/src/modules/file/controller/file_controller.rs:140+`（read_file/write_file/mkdir/touch/rename/move/copy/delete/download 全套）
- **证据**：`file_service.rs:normalize_path` 只解 `..`，未校验路径必须落在 `run_dir/files/` 或用户上传根下
- **攻击**：`POST /api/files/write body {"path":"C:\\Windows\\System32\\drivers\\etc\\hosts"}` 直接覆盖 hosts
- **改法**：在 `file_service.rs` 入口加 `fn in_workspace(p: &Path) -> bool` 用 `canonicalize().starts_with(run_dir.join("files"))`；非 root 用户再叠加 ACL
- **风险等级**：🔴 高（认证后即可越权文件系统）

### 2. CORS 全开
- **位置**：`backend/src/app/router.rs:199`
- **证据**：`CorsLayer::permissive()` 允许任意 Origin/Method/Header
- **改法**：改为 `CorsLayer::new().allow_origin([...白名单...]).allow_methods([Get, Post, Put, Delete])`，默认 deny
- **风险等级**：🔴 高

### 3. POST handler 漏 `#[audit_log]`（合规盲点）
- **位置**：`backend/src/modules/site/controller/cert_controller.rs:20 apply_certificate`、`:31 renew_certificate`
- **证据**：两个 POST 跑 acme.sh + 改 nginx 配置，是高敏操作，但无审计记录
- **改法**：两个 handler 加 `#[audit_log(module = "certificate", action = "...", capture = req)]`
- **风险等级**：🔴 高

### 4. settings_controller 静默吞错 × 8
- **位置**：`backend/src/modules/settings/controller/settings_controller.rs:130,135,140,145,150,155,160,177`
- **证据**：6 处 `let _ = param_service::update_param(...)` + 1 处 `let _ = std::fs::write(&config_path, ...)`
- **后果**：写 nginx 配置失败用户无感知 → 下次 reload 用旧值或崩
- **改法**：用 `?` 上抛，在 handler 顶部用 audit ctx 写入 error 后返回 500
- **风险等级**：🔴 高

### 5. backup_controller 静默吞错
- **位置**：`backend/src/modules/backup/controller/backup_controller.rs:63`
- **证据**：`let _ = crate::modules::common::nginx::write_site_config(...)`；restore 失败用户看不到
- **改法**：同样上抛
- **风险等级**：🔴 高

### 6. service/controller 层混入 SQL（违反 CLAUDE.md 约束）
- **位置**：
  - `backend/src/modules/auth/controller/auth_controller.rs:33,113,156` — `sqlx::query_as::<_, User>(...)`
  - `backend/src/modules/sys/service/user_service.rs:271` — `sqlx::query("UPDATE sys_user SET disabled = ? WHERE id = ?")`
  - `backend/src/modules/log/dao/log_dao.rs:96/155/157/208/250` — `format!("...{}...", wheres.join(" AND "))` 动态 SQL（bind 安全但违反模式）
- **证据**：CLAUDE.md 明文 "Service 层直接写 SQL 字符串…Service 不能有 sqlx::query"
- **改法**：auth_controller 三处迁到 `sys/dao/user_dao.rs`；user_service.rs:271 迁到 `sys/dao/user_dao.rs::batch_set_disabled`
- **风险等级**：🟡 中（架构债；也是后续回归隐患）

---

## 🟡 第二轮新发现（MED 性能/可维护性）

### 7. AppState.get_config 热路径深拷贝
- **位置**：`backend/src/app/state.rs:37-39`
- **证据**：`pub fn get_config(&self) -> AppConfig { self.config.read().clone() }` — 每次调用 clone 整个 AppConfig（含所有 String）
- **改法**：传引用 `pub fn config(&self) -> parking_lot::RwLockReadGuard<'_, AppConfig>`；或一次性快照读锁返回 `&AppConfig`
- **风险等级**：🟡 中（dashboard 10s tick 都调用，影响 dashboard endpoint 热路径）

### 8. nginx 下载无 SHA256 校验
- **位置**：`backend/src/modules/common/nginx/process.rs:307,319`
- **证据**：`https://nginx.org/download/nginx-{}.zip` 直接 `curl -L` 下载，没校验 sha256
- **改法**：至少校验下载产物的 SHA256 或签名；或固定使用 `libs/nginx/windows/` 本地包（已有回退）
- **风险等级**：🟡 中（中间人篡改 → 执行恶意 nginx）

### 9. audit middleware 跳过列表过宽
- **位置**：`backend/src/modules/common/audit/middleware.rs:188-203`
- **证据**：`uri.contains("/list")` 会误放过任何 path 里有 "list" 的 POST；`uri.contains("/log/")` 放过任何 path 含 `/log/` 的写操作
- **改法**：改成 prefix 表 + HTTP method 组合白名单，POST 不应放行
- **风险等级**：🟢 低（写操作路径里带 "list"/"log" 字符串的极少）

### 10. `as any` 滥用 × 11
- **位置**：`frontend/src/hooks/useCrud.ts:11/15/17/90/97/106/113/124`（最严重）、`components/OnTable/types.ts`（组件库通用）、`views/sites/index.vue:34`、其他零散
- **改法**：`useCrud<T, Create = Partial<T>, Update = Partial<T>>` 泛型化；OnTable 通用 wrapper 可保留但加 `ponytail:` 注释
- **风险等级**：🟡 中

### 11. `Site` 类型双声明（编译不报错但运行时错位）
- **位置**：`frontend/src/api/sites/type.ts:1` 与 `frontend/src/views/sites/types.ts:1`
- **证据**：两套 interface 字段不一致时编译不报错但运行时表现不同；`BackupFile` 在 `views/sites/types.ts:53` 与 `SiteBackupDialog.vue:53` 重复
- **改法**：删除 `views/sites/types.ts`，从 `api/sites/type.ts` 统一 export；删除 `SiteBackupDialog.vue:53`
- **风险等级**：🟡 中

### 12. `.gitignore` 缺项
- **位置**：根 `.gitignore`
- **证据**：
  - `backend/*.db` ✅ 但 `backend/data.db-wal` / `backend/data.db-shm` 没排除
  - `*.log` 没排除
  - `.codegraph/daemon.log` 应排除
- **改法**：
  ```
  backend/data.db-*
  *.log
  .codegraph/
  ```
- **风险等级**：🟡 中

---

## ⏳ 第一轮遗留（P1.9b + P2）

### P1.9b dto 拆分（最大）
**位置**：`backend/src/modules/common/dto/mod.rs` 518 行

**改法**：按业务拆 5 文件，mod.rs 用 `pub use` 重导出保兼容：
- `mod.rs`（留通用）：`ApiResponse`、`PagedResult`、`LoginRequest/Response`、`ChangePassword/Username`
- `site.rs`：`CreateSiteRequest`、`UpdateSiteRequest`、`DeleteSiteRequest`、`ApplyCertRequest`、`NginxTestResult`
- `upstream.rs`：`Create/UpdateUpstreamRequest`、`UpstreamServerRequest`、`Create/UpdateAccessRuleRequest`、`Create/UpdateTemplateRequest`
- `dashboard.rs`：`DashboardData`、`LogResponse`
- `rbac.rs`：`MenuNode`、`RbacInfo`、`PageQuery`、`UserQuery`、`Upsert*` 系列（User/Role/Dept/Post/Menu）、`SetRoleMenusRequest`
- `sys.rs`：`I18nKv`、`UpsertI18nRequest`、`UpsertDictRequest`、`UpsertDictItemRequest`、`DictWithItems`、`UpsertParamRequest`、`BatchDeleteFilesRequest`、`UploadFileResponse`

**风险等级**：🟢 低。`file_dto.rs` 已有先例。`pub use` 保兼容。

### P2 清理（小改，一条 commit 搞定）

| # | 位置 | 动作 |
|---|---|---|
| 2-1 | `backend/Cargo.toml` | 删 `thiserror = "2"`（代码无自定义 enum 错误类型） |
| 2-2 | `frontend/vite.config.ts` | `optimizeDeps.include` 加 `monaco-editor/esm/vs/editor/editor.worker` |
| 2-3 | `backend/src/modules/sys/service/user_service.rs:261-275` | `batch_set_disabled` 串行 N 次 SQL → 一次 `UPDATE ... WHERE id IN (...)` |
| 2-4 | `backend/src/modules/sys/service/{user,dept}_service.rs:78,69` | `parent_id.unwrap()` 改为 `unwrap_or(0_i64)` |

---

## 📋 仓库状态速览

```
最近 16 个 commit（master 分支）：
cf21fde refactor(site): site_controller 按职责拆 3 文件
c3e9cf1 refactor(seed): seed_menu 默认菜单搬到 menu.json
93b5956 fix(style): EP dark css-vars 顺序
718dd52 fix(release): version.ps1 中文注释 + UTF-8 BOM
5643bfb chore(release): version.ps1 全量同步 + dry-run
f0c1ca1 perf(backup): create_backup 改 spawn_blocking
6862e16 refactor(frontend): ElementPlus 按需引入
77c7165 fix(unwrap): IO 路径 unwrap 全部消除
9cf2020 fix(unwrap): SQL 路径 unwrap 全部消除
95f230d refactor(lock): std::Mutex → parking_lot
cee6a96 docs(claude): 合并 AGENTS.md 关键细节

未推送：本地领先 origin/master 11 commits
```

---

## 🔧 明天开工建议路径

### 第一档（必修安全/合规，按顺序）
1. **#1 文件 API 路径白名单**（30 分钟）
2. **#2 CORS 白名单**（10 分钟）
3. **#3 cert_controller 补 audit**（5 分钟）
4. **#4 + #5 静默吞错改 `?`**（20 分钟）
5. **#6 SQL 越层迁 dao**（30 分钟）

### 第二档（性能/可维护性）
6. **#7 get_config 改引用**（15 分钟）
7. **#8 nginx SHA256**（30 分钟，加下载后校验）
8. **#9-#11 + .gitignore**（20 分钟）

### 第三档（结构整理）
9. **P1.9b dto 拆分**（45 分钟，机械拆分）
10. **P2 五条小改**（15 分钟一并提交）

跑 `pnpm run lint && cargo check && pnpm run build` 全验证。

---

*生成于 2026-07-13*
*第二轮体检：探查架构债与静默失败*