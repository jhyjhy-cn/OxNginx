# 前端国际化说明

本目录的 `locales/{zh-CN,en-US}.ts` 是**前端兜底**翻译文件。
运行时由后端 `sys_i18n` 表的翻译**覆盖**这些静态文件。

## 与后端的分工

| 谁负责 | 文件 | namespace |
|---|---|---|
| 前端 ts（本目录） | `locales/zh-CN.ts` / `locales/en-US.ts` | `common` / `layout` / `login` / `forbidden` |
| 后端 DB（权威） | `backend/src/modules/common/database/language/*.json` seed → `sys_i18n` 表 | `sys.<module>.*` |

**后端是权威源**。改业务文本在后台 → 国际化管理页面在线改，
**不需要**改后端 JSON 或前端 ts。

前端 ts 只在以下场景会被实际渲染：
- 后端 `sys_i18n` 表里**没有**该 key（首次启动未 seed、locale 缺失、key 被删）
- 后端 API 暂未拉取到（启动瞬间）

## 4 个 frontend namespace 的作用

| namespace | 说明 |
|---|---|
| `common.*` | 全局通用按钮、文案（添加/编辑/删除/确定/取消/成功/失败/导出/…） |
| `layout.*` | 主布局（首页、修改密码、修改账号、退出登录） |
| `login.*` | 登录页、初始化页（用户名/密码、协议、ICP） |
| `forbidden.*` | 403 无权限拦截页 |

**所有页面级 key**（`sys.menu.*` / `sys.rbac.*` / `sys.sites.*` …）**都属于后端**，
前端代码里 `t('sys.xxx.yyy')` 只是查表，不在本目录定义。

## 加载流程

```
1. createI18n() 用本目录 ts 初始化 vue-i18n（兜底）
2. App.vue / useI18n.ts 调 GET /api/rbac/i18n/messages?locale=zh-CN
3. 后端从 sys_i18n 查所有 (zh-CN, key, value) flat list
4. 前端 mergeI18nMessages() 把 flat 嵌套成树，深度合并到 vue-i18n
5. 业务 key 走 t('sys.menu.dashboard') 命中后端翻译
   兜底 key 走 t('common.add') 命中 ts 翻译
```

## 怎么改

| 想改 | 步骤 |
|---|---|
| `common.*` 通用按钮文案 | 改本目录 `*.ts`（前端），重新 build |
| `layout.*` / `login.*` / `forbidden.*` | 改本目录 `*.ts`，重新 build |
| 业务 key（菜单、表头、提示…） | 改后台 → 国际化管理 |

**注意**：本目录 ts 一旦 build 后改完要重新构建；后端改完即生效（无需重启前端）。

## 校验

- `pnpm run build` 通过 = ts 语法、类型、import 全 OK
- vue-tsc 会校验 `t('xxx.yyy')` 的 key 是否存在于某个 namespace（嵌套对象）
  缺失会报错（但 `t('sys.xxx')` 因为是后端动态塞的，编译期不会校验）
