# 国际化语言文件说明

本目录存放**后端**的种子 i18n 文本，编译期通过 `include_str!` 嵌入二进制，
首次运行时由 `seed_i18n` 写入 `sys_i18n` 表。

## 分工规则

**前端**  `frontend/src/i18n/locales/{zh-CN,en-US}.ts`
- 只保留 **基础设施类**：登录页、布局、403 拦截、全局通用按钮
- 固定 4 个 namespace：`common` / `layout` / `login` / `forbidden`
- 改这些 → 改前端 ts，不需要重新建库

**后端**  `language/{zh-CN,en-US}.json`（本目录）
- 所有 **页面级 / 业务级** 文本都在这里
- 统一以 `sys.` 前缀开头，例如 `sys.menu.dashboard` / `sys.rbac.createUser` / `sys.user.phone`
- 通过后台 **"国际化管理"** 页面在线编辑，写入 `sys_i18n` 表
- 首次启动时由 seed 灌入；后续编辑不依赖 JSON

## 加载流程

```
JSON (编译期)              运行时
  include_str!  ──>  seed_i18n  ──>  sys_i18n 表
                                       │
前端 t('sys.xx.yy')  ◀── mergeI18nMessages ◀── GET /api/rbac/i18n/messages?locale=zh-CN
                       （DB 优先，ts 兜底）
```

## 怎么改文本

| 改哪儿 | 步骤 |
|---|---|
| 按钮 / 通用文案 | 改 `common.*`（前端 ts）或后台 → 国际化管理 |
| 登录 / 布局 / 403 | 改前端 `login.*` / `layout.*` / `forbidden.*` |
| 菜单名 | 改 `sys.menu.*`（JSON） 或 后台 → 国际化管理 |
| 业务页面 | 改对应 `sys.<module>.*`（JSON） 或 后台 → 国际化管理 |
| 加新页面 | 加 `sys.<new>.*` 到 JSON，重新 build；后台管理页可在线补翻译 |

## 命名空间约定

- 前端 ts 兜底：保持原 namespace `common` / `layout` / `login` / `forbidden`
- 后端业务 key：必须以 `sys.` 开头，按模块分组
  - `sys.menu.*` 菜单与侧边栏
  - `sys.rbac.*` 用户/角色/部门/岗位/菜单/字典/国际化 等管理页
  - `sys.user.*` 用户表头字段
  - `sys.dashboard.*` 仪表盘
  - `sys.sites.*` / `sys.siteDetail.*` 站点
  - `sys.ssl.*` / `sys.access.*` / `sys.templates.*` / `sys.upstreams.*` / `sys.logs.*` / `sys.config.*` / `sys.settings.*` / `sys.files.*` / `sys.terminal.*`
  - `sys.log.*` 登录/操作日志列表列名
  - `sys.dict.*` 字典管理

## 校验

JSON 用 `include_str!` 嵌入，启动时 `serde_json::from_str` 解析。
语法错或漏 `,` → cargo build 通过但启动时 panic；**build 通过 = 编译期无错**，
**首次启动看日志确认 seed_i18n 无错**。
