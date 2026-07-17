-- OxNginx 数据库初始化 DDL
-- 由 backend/src/modules/common/database/mod.rs 通过 include_str! 嵌入编译
-- 字段命名规范：created_by/created_at/updated_by/updated_at/dept_id/remark/sort/version
-- is_deleted: 0=未删, 1=已删（仅 sys_users）
-- 状态字段 status：1=启用 0=禁用

-- =====================================================================
-- 用户与认证
-- =====================================================================

-- 用户表：系统登录账号，存密码哈希、所属部门/岗位、个人资料等
CREATE TABLE IF NOT EXISTS sys_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    username TEXT NOT NULL UNIQUE,                        -- 登录用户名
    password TEXT NOT NULL,                               -- 密码（Argon2 哈希）
    nickname TEXT,                                        -- 昵称
    phone TEXT,                                           -- 手机号
    email TEXT,                                           -- 邮箱
    gender TEXT,                                          -- 性别：male/female/secret
    remark TEXT,                                          -- 备注
    dept_id INTEGER,                                      -- 所属部门 ID
    post_id INTEGER,                                      -- 所属岗位 ID
    disabled INTEGER NOT NULL DEFAULT 0,                  -- 是否禁用：0=否 1=是
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 用户 Token：会话凭证，过期后清理
CREATE TABLE IF NOT EXISTS sys_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    token TEXT NOT NULL UNIQUE,                           -- Token 字符串
    user_id INTEGER NOT NULL,                             -- 所属用户 ID
    username TEXT NOT NULL,                               -- 冗余用户名（便于审计）
    ip TEXT,                                              -- 登录 IP
    os TEXT,                                              -- 操作系统
    browser TEXT,                                         -- 浏览器
    user_agent TEXT,                                      -- 原始 UA
    expires_at DATETIME NOT NULL,                         -- 过期时间
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    FOREIGN KEY (user_id) REFERENCES sys_users(id) ON DELETE CASCADE
);

-- =====================================================================
-- 站点相关（site_ 前缀，详见 site 模块）
-- =====================================================================

-- 站点表：每个站点对应一个 Nginx server 块
CREATE TABLE IF NOT EXISTS site_sites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    name TEXT NOT NULL,                                   -- 站点名称
    server_name TEXT NOT NULL,                            -- 域名
    listen TEXT NOT NULL DEFAULT '80',                    -- 监听端口
    ssl INTEGER NOT NULL DEFAULT 0,                       -- 是否启用 HTTPS：0=否 1=是
    certificate_path TEXT,                                -- 证书文件路径
    key_path TEXT,                                        -- 私钥文件路径
    proxy_pass TEXT,                                      -- 反向代理目标地址
    root_path TEXT,                                       -- 站点根目录（静态资源）
    config TEXT,                                          -- 自定义 Nginx 配置片段
    remark TEXT,                                          -- 备注
    expire_time DATETIME,                                 -- 站点到期时间
    rewrite_rules TEXT,                                   -- URL 重写规则
    redirect_rules TEXT,                                  -- 重定向规则
    hotlink_config TEXT,                                  -- 防盗链配置
    log_access_path TEXT,                                 -- 访问日志路径
    log_error_path TEXT,                                  -- 错误日志路径
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 证书表：站点使用的 SSL 证书，支持自动续期
CREATE TABLE IF NOT EXISTS site_certificates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    domain TEXT NOT NULL UNIQUE,                          -- 域名
    issuer TEXT,                                          -- 颁发者
    expire_time DATETIME,                                 -- 过期时间
    cert_path TEXT,                                       -- 证书文件路径
    key_path TEXT,                                        -- 私钥文件路径
    auto_renew INTEGER NOT NULL DEFAULT 1,                -- 是否自动续期：0=否 1=是
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 站点配置备份：每次变更前的快照，用于回滚
CREATE TABLE IF NOT EXISTS site_backups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    site_id INTEGER,                                      -- 所属站点 ID
    version INTEGER NOT NULL DEFAULT 1,                   -- 备份版本号（非乐观锁）
    config TEXT NOT NULL,                                 -- 备份的 Nginx 配置内容
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 更新时间
    FOREIGN KEY (site_id) REFERENCES site_sites(id)
);

-- 负载均衡 upstream 定义
CREATE TABLE IF NOT EXISTS site_upstreams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    name TEXT NOT NULL UNIQUE,                            -- upstream 名称
    method TEXT NOT NULL DEFAULT 'round_robin',           -- 负载均衡算法
    keepalive INTEGER DEFAULT 32,                         -- 保持连接数
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- upstream 下的真实服务器节点
CREATE TABLE IF NOT EXISTS site_upstream_servers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    upstream_id INTEGER NOT NULL,                         -- 所属 upstream ID
    address TEXT NOT NULL,                                -- 服务器地址（IP:PORT）
    weight INTEGER DEFAULT 1,                             -- 权重
    max_fails INTEGER DEFAULT 3,                          -- 最大失败次数
    fail_timeout TEXT DEFAULT '30s',                      -- 失败超时时间
    backup INTEGER DEFAULT 0,                             -- 是否备用服务器：0=否 1=是
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    FOREIGN KEY (upstream_id) REFERENCES site_upstreams(id) ON DELETE CASCADE
);

-- 访问控制规则：黑/白名单、限流等
CREATE TABLE IF NOT EXISTS site_access_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    site_id INTEGER,                                      -- 所属站点 ID
    rule_type TEXT NOT NULL,                              -- 规则类型
    value TEXT NOT NULL,                                  -- 规则值（IP/CIDR/UA 等）
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 更新时间
    FOREIGN KEY (site_id) REFERENCES site_sites(id) ON DELETE CASCADE
);

-- 站点配置模板：用于批量生成站点
CREATE TABLE IF NOT EXISTS site_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    name TEXT NOT NULL UNIQUE,                            -- 模板名称
    remark TEXT,                                          -- 备注
    config TEXT NOT NULL,                                 -- 模板配置内容
    variables TEXT,                                       -- 模板变量定义（JSON）
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 文件备注：给文件管理器中的路径打备注
CREATE TABLE IF NOT EXISTS file_notes (
    path TEXT PRIMARY KEY,                                -- 文件路径（主键）
    note TEXT NOT NULL,                                   -- 备注内容
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 反向代理：站点下的子路径代理规则
CREATE TABLE IF NOT EXISTS site_reverse_proxies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    site_id INTEGER NOT NULL,                             -- 所属站点 ID
    name TEXT NOT NULL,                                   -- 代理名称
    proxy_dir TEXT NOT NULL DEFAULT '/',                  -- 代理路径前缀
    target_url TEXT NOT NULL,                             -- 目标 URL
    cache INTEGER NOT NULL DEFAULT 0,                     -- 是否启用缓存：0=否 1=是
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    dept_id INTEGER,                                      -- 归属部门 ID
    is_deleted INTEGER NOT NULL DEFAULT 0,                -- 逻辑删除：0=未删 1=已删
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 更新时间
    FOREIGN KEY (site_id) REFERENCES site_sites(id) ON DELETE CASCADE
);

-- =====================================================================
-- RBAC（角色权限）
-- =====================================================================

-- 角色：权限的聚合单位，绑定菜单即可控制前端可见性
CREATE TABLE IF NOT EXISTS sys_roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    code TEXT NOT NULL UNIQUE,                            -- 角色代码（英文唯一标识）
    name TEXT NOT NULL,                                   -- 角色名称（中文友好）
    remark TEXT,                                          -- 备注
    dept_id INTEGER,                                      -- 归属部门 ID（可选）
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 部门：树形组织结构
CREATE TABLE IF NOT EXISTS sys_depts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    parent_id INTEGER,                                    -- 父部门 ID
    name TEXT NOT NULL,                                   -- 部门名称
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    remark TEXT,                                          -- 备注
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 岗位：职位定义，隶属于部门
CREATE TABLE IF NOT EXISTS sys_posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    code TEXT NOT NULL UNIQUE,                            -- 岗位代码
    name TEXT NOT NULL,                                   -- 岗位名称
    dept_id INTEGER,                                      -- 所属部门 ID
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    remark TEXT,                                          -- 备注
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 菜单/权限树：type=1 目录 / 2 菜单 / 3 按钮；permission 字段为按钮级权限标识
CREATE TABLE IF NOT EXISTS sys_menus (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    parent_id INTEGER,                                    -- 父菜单 ID
    name TEXT NOT NULL,                                   -- 菜单名称（树内唯一）
    title TEXT NOT NULL,                                  -- 菜单标题（用于 i18n key）
    icon TEXT,                                            -- 图标
    path TEXT,                                            -- 路由路径
    component TEXT,                                       -- 前端组件路径
    type INTEGER NOT NULL,                                -- 菜单类型：1=目录 2=菜单 3=按钮
    permission TEXT,                                      -- 权限标识（如 sys:user:add）
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    remark TEXT,                                          -- 备注
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 用户-角色关联
CREATE TABLE IF NOT EXISTS sys_user_roles (
    user_id INTEGER NOT NULL,                             -- 用户 ID
    role_id INTEGER NOT NULL,                             -- 角色 ID
    PRIMARY KEY (user_id, role_id)
);

-- 角色-菜单（权限）关联
CREATE TABLE IF NOT EXISTS sys_role_menus (
    role_id INTEGER NOT NULL,                             -- 角色 ID
    menu_id INTEGER NOT NULL,                             -- 菜单/按钮 ID
    PRIMARY KEY (role_id, menu_id)
);

-- =====================================================================
-- ponytail: 暂不使用的国际化建表（改前端 ts 兜底）。需要恢复时取消注释 + 启用路由/seed。
-- =====================================================================
/*
-- 国际化翻译条目：(locale, key) 唯一
CREATE TABLE IF NOT EXISTS sys_i18n (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    locale TEXT NOT NULL,                                 -- 语言代码（如 zh-CN/en-US）
    key TEXT NOT NULL,                                    -- 翻译键
    value TEXT NOT NULL,                                  -- 翻译值
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    remark TEXT,                                          -- 备注
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 更新时间
    UNIQUE(locale, key)
);
*/

-- =====================================================================
-- 字典
-- =====================================================================
CREATE TABLE IF NOT EXISTS sys_dict (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    name TEXT NOT NULL,                                   -- 字典名称（中文友好）
    code TEXT NOT NULL UNIQUE,                            -- 字典代码（英文唯一标识）
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 字典项：字典下的具体可选值
CREATE TABLE IF NOT EXISTS sys_dict_item (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    dict_id INTEGER NOT NULL,                             -- 所属字典 ID
    label TEXT NOT NULL,                                  -- 显示文本
    value TEXT NOT NULL,                                  -- 实际值
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    status INTEGER NOT NULL DEFAULT 1,                    -- 状态：1=启用 0=禁用
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    remark TEXT,                                          -- 备注
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 更新时间
    FOREIGN KEY (dict_id) REFERENCES sys_dict(id) ON DELETE CASCADE
);

-- =====================================================================
-- 系统资源
-- =====================================================================

-- 系统参数：键值对形式存储的可配置项
CREATE TABLE IF NOT EXISTS sys_params (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    key TEXT NOT NULL UNIQUE,                             -- 参数标识（英文唯一）
    value TEXT,                                           -- 参数值
    name TEXT NOT NULL,                                   -- 参数名（中文友好）
    group_code TEXT NOT NULL DEFAULT 'default',           -- 分组编码
    remark TEXT,                                          -- 备注
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- 系统文件：上传到系统的附件资源
CREATE TABLE IF NOT EXISTS sys_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    name TEXT NOT NULL UNIQUE,                            -- 服务器文件名（uuid + 后缀）
    original_name TEXT NOT NULL,                          -- 原始文件名
    suffix TEXT NOT NULL,                                 -- 后缀（小写）
    size INTEGER NOT NULL DEFAULT 0,                      -- 字节数
    mime_type TEXT,                                       -- Content-Type
    md5 TEXT,                                             -- 文件 MD5
    path TEXT NOT NULL,                                   -- 相对路径（不含域名）
    provider TEXT NOT NULL DEFAULT 'local',               -- 存储服务商
    dept_id INTEGER,                                      -- 归属部门 ID
    remark TEXT,                                          -- 备注
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);

-- =====================================================================
-- 日志（仅插入，不更新）
-- =====================================================================

-- 操作日志：API 请求/操作流水
CREATE TABLE IF NOT EXISTS sys_operation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    trace_id TEXT,                                        -- 链路追踪 ID
    username TEXT NOT NULL,                               -- 操作人
    module TEXT,                                          -- 模块
    action TEXT NOT NULL,                                 -- 操作描述
    method TEXT,                                          -- HTTP 方法
    uri TEXT,                                             -- 请求路径
    ip TEXT,                                              -- 客户端 IP
    status INTEGER NOT NULL DEFAULT 1,                    -- 操作结果：1=成功 0=失败
    cost_ms INTEGER,                                      -- 耗时（毫秒）
    duration_ms INTEGER,                                  -- 持续时间（毫秒）
    request_body TEXT,                                    -- 请求体
    response_body TEXT,                                   -- 响应体
    error_msg TEXT,                                       -- 错误信息
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 创建时间
);

-- 登录日志：登录/登出事件
CREATE TABLE IF NOT EXISTS sys_login_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    username TEXT NOT NULL,                               -- 用户名
    ip TEXT,                                              -- 客户端 IP
    os TEXT,                                              -- 操作系统
    browser TEXT,                                         -- 浏览器
    user_agent TEXT,                                      -- 完整 UserAgent
    type INTEGER NOT NULL DEFAULT 1,                      -- 类型：1=登录 0=登出
    status INTEGER NOT NULL DEFAULT 1,                    -- 结果：1=成功 0=失败
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 创建时间
);

-- =====================================================================
-- 数据库管理（外部数据库连接元数据 + 状态探测）
-- =====================================================================

-- 数据库连接：用户配置的外部数据源,运行时探测用
CREATE TABLE IF NOT EXISTS dbm_databases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,                 -- 主键
    type TEXT NOT NULL,                                   -- 类型：redis / sqlite
    name TEXT NOT NULL UNIQUE,                            -- 展示名,唯一
    host TEXT,                                            -- redis 主机
    port INTEGER,                                         -- redis 端口
    username TEXT,                                        -- redis ACL 用户(可选)
    password TEXT,                                        -- redis 密码(明文,service 层返回前 mask)
    db_name TEXT,                                         -- sqlite 文件名 / redis db index
    db_path TEXT,                                         -- sqlite 绝对路径(可选,默认走项目 db)
    enabled INTEGER NOT NULL DEFAULT 1,                   -- 启用：1=是 0=否
    sort INTEGER NOT NULL DEFAULT 0,                      -- 排序
    remark TEXT,                                          -- 备注
    version INTEGER NOT NULL DEFAULT 0,                   -- 乐观锁版本号
    created_by INTEGER,                                   -- 创建人 user_id
    updated_by INTEGER,                                   -- 修改人 user_id
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,        -- 创建时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP         -- 更新时间
);
CREATE INDEX IF NOT EXISTS idx_dbm_databases_type ON dbm_databases(type);