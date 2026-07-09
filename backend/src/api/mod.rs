pub mod access_api;
pub mod auth_api;
pub mod backup_api;
pub mod config_api;
pub mod dashboard_api;
pub mod dashboard_ws;
pub mod file_api;
pub mod log_api;
pub mod nginx_api;

// RBAC 子模块
pub mod sys_user_api;
pub mod sys_role_api;
pub mod sys_dept_api;
pub mod sys_post_api;
pub mod sys_menu_api;
pub mod sys_dict_api;
pub mod sys_i18n_api;
pub mod rbac_api; // 仅保留共享接口

pub mod reverse_proxy_api;
pub mod settings_api;
pub mod site_api;
pub mod site_backup_api;
pub mod template_api;
pub mod terminal_api;
pub mod upstream_api;
