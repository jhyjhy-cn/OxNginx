use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::modules;
use crate::modules::common::audit;
use crate::modules::common::middleware;
use super::state::AppState;

/// 构建应用路由
pub fn build(state: AppState) -> Router {
    // 公开路由（无需认证）
    let public_routes = Router::new()
        .route("/api/login", post(modules::auth::controller::auth_controller::login))
        .route("/api/logout", post(modules::auth::controller::auth_controller::logout))
        .route("/api/setup", post(modules::auth::controller::auth_controller::setup))
        .route("/api/setup/status", get(modules::auth::controller::auth_controller::setup_status))
        .route("/api/auth/public-key", get(modules::auth::controller::auth_controller::get_public_key))
        .route("/api/terminal/ws", get(modules::system::controller::terminal_controller::terminal_ws))
        .route("/api/dashboard/ws", get(modules::dashboard::controller::dashboard_ws::dashboard_ws));

    // 需要认证的路由
    let protected_routes = Router::new()
        .route("/api/dashboard", get(modules::dashboard::controller::dashboard_controller::get_dashboard))
        .route("/api/sites", get(modules::site::controller::site_controller::list_sites))
        .route("/api/sites/with-certs", get(modules::site::controller::site_controller::list_sites_with_certs))
        .route("/api/sites", post(modules::site::controller::site_controller::create_site))
        .route("/api/sites/{id}", get(modules::site::controller::site_controller::get_site))
        .route("/api/sites/{id}", put(modules::site::controller::site_controller::update_site))
        .route("/api/sites/{id}", delete(modules::site::controller::site_controller::delete_site))
        .route("/api/sites/{id}/deploy-ssl", post(modules::site::controller::site_controller::deploy_ssl))
        .route("/api/sites/batch/enable", post(modules::site::controller::site_controller::batch_enable))
        .route("/api/sites/batch/disable", post(modules::site::controller::site_controller::batch_disable))
        .route("/api/sites/batch/delete", post(modules::site::controller::site_controller::batch_delete))
        // 反向代理
        .route("/api/sites/{id}/proxies", get(modules::site::controller::reverse_proxy_controller::list_proxies))
        .route("/api/sites/{id}/proxies", post(modules::site::controller::reverse_proxy_controller::create_proxy))
        .route("/api/proxies/{id}", put(modules::site::controller::reverse_proxy_controller::update_proxy))
        .route("/api/proxies/{id}", delete(modules::site::controller::reverse_proxy_controller::delete_proxy))
        .route("/api/certificates", get(modules::site::controller::cert_controller::list_certificates))
        .route("/api/certificate/apply", post(modules::site::controller::cert_controller::apply_certificate))
        .route("/api/certificate/renew", post(modules::site::controller::cert_controller::renew_certificate))
        .route("/api/change-password", post(modules::auth::controller::auth_controller::change_password))
        .route("/api/change-username", post(modules::auth::controller::auth_controller::change_username))
        .route("/api/nginx/test", post(modules::nginx::controller::nginx_controller::test_config))
        .route("/api/nginx/reload", post(modules::nginx::controller::nginx_controller::reload))
        .route("/api/nginx/status", get(modules::nginx::controller::nginx_controller::status))
        .route("/api/nginx/start", post(modules::nginx::controller::nginx_controller::start))
        .route("/api/nginx/stop", post(modules::nginx::controller::nginx_controller::stop))
        .route("/api/nginx/restart", post(modules::nginx::controller::nginx_controller::restart))
        .route("/api/nginx/install", post(modules::nginx::controller::nginx_controller::install))
        .route("/api/log/access", get(modules::log::controller::log_controller::access_log))
        .route("/api/log/error", get(modules::log::controller::log_controller::error_log))
        .route("/api/log/operation", get(modules::log::controller::log_controller::list_operation_logs))
        .route("/api/log/operation/export", get(modules::log::controller::log_controller::export_operation_logs))
        .route("/api/log/login", get(modules::log::controller::log_controller::list_login_logs))
        .route("/api/log/login/export", get(modules::log::controller::log_controller::export_login_logs))
        .route("/api/backups/{id}", get(modules::backup::controller::backup_controller::list_backups))
        .route("/api/backups/{id}", post(modules::backup::controller::backup_controller::create_backup))
        .route("/api/backups/{id}", delete(modules::backup::controller::backup_controller::delete_backup))
        .route("/api/backups/restore/{id}", post(modules::backup::controller::backup_controller::restore_backup))
        .route("/api/backups/diff", post(modules::backup::controller::backup_controller::diff_backups))
        .route("/api/sites/{id}/backups", get(modules::site::controller::site_backup_controller::list_site_backups))
        .route("/api/sites/{id}/backups", post(modules::site::controller::site_backup_controller::create_site_backup))
        .route("/api/sites/{id}/backups/{filename}", delete(modules::site::controller::site_backup_controller::delete_site_backup))
        .route("/api/sites/{id}/backups/{filename}/download", get(modules::site::controller::site_backup_controller::download_site_backup))
        .route("/api/sites/{id}/backups/batch-delete", post(modules::site::controller::site_backup_controller::batch_delete_site_backups))
        .route("/api/config/main", get(modules::settings::controller::config_controller::get_main_config))
        .route("/api/config/main", put(modules::settings::controller::config_controller::save_main_config))
        .route("/api/config/files", get(modules::settings::controller::config_controller::list_config_files))
        .route("/api/config/file/{name}", get(modules::settings::controller::config_controller::get_site_config))
        .route("/api/config/file/{name}", put(modules::settings::controller::config_controller::save_site_config))
        .route("/api/config/file/{name}/toggle", post(modules::settings::controller::config_controller::toggle_site_config))
        .route("/api/config/file/{name}", delete(modules::settings::controller::config_controller::delete_site_config))
        .route("/api/upstreams", get(modules::site::controller::upstream_controller::list_upstreams))
        .route("/api/upstreams", post(modules::site::controller::upstream_controller::create_upstream))
        .route("/api/upstreams/{id}", get(modules::site::controller::upstream_controller::get_upstream))
        .route("/api/upstreams/{id}", put(modules::site::controller::upstream_controller::update_upstream))
        .route("/api/upstreams/{id}", delete(modules::site::controller::upstream_controller::delete_upstream))
        .route("/api/access-rules", get(modules::site::controller::access_controller::list_rules))
        .route("/api/access-rules", post(modules::site::controller::access_controller::create_rule))
        .route("/api/access-rules/{id}", get(modules::site::controller::access_controller::get_rule))
        .route("/api/access-rules/{id}", put(modules::site::controller::access_controller::update_rule))
        .route("/api/access-rules/{id}", delete(modules::site::controller::access_controller::delete_rule))
        .route("/api/templates", get(modules::site::controller::template_controller::list_templates))
        .route("/api/templates", post(modules::site::controller::template_controller::create_template))
        .route("/api/templates/{id}", get(modules::site::controller::template_controller::get_template))
        .route("/api/templates/{id}", put(modules::site::controller::template_controller::update_template))
        .route("/api/templates/{id}", delete(modules::site::controller::template_controller::delete_template))
        .route("/api/templates/{id}/preview", post(modules::site::controller::template_controller::preview_template))
        .route("/api/settings", get(modules::settings::controller::settings_controller::get_settings))
        .route("/api/settings", put(modules::settings::controller::settings_controller::update_settings))
        .route("/api/files/list", post(modules::file::controller::file_controller::list_files))
        .route("/api/files/roots", get(modules::file::controller::file_controller::list_roots))
        .route("/api/files/read", post(modules::file::controller::file_controller::read_file))
        .route("/api/files/write", post(modules::file::controller::file_controller::write_file))
        .route("/api/files/mkdir", post(modules::file::controller::file_controller::mkdir))
        .route("/api/files/touch", post(modules::file::controller::file_controller::touch))
        .route("/api/files/rename", post(modules::file::controller::file_controller::rename))
        .route("/api/files/move", post(modules::file::controller::file_controller::move_file))
        .route("/api/files/copy", post(modules::file::controller::file_controller::copy_file))
        .route("/api/files/delete", delete(modules::file::controller::file_controller::delete_file))
        .route("/api/files/chmod", post(modules::file::controller::file_controller::chmod))
        .route("/api/files/compress", post(modules::file::controller::file_controller::compress))
        .route("/api/files/extract", post(modules::file::controller::file_controller::extract))
        .route("/api/files/note", post(modules::file::controller::file_controller::save_note))
        .route("/api/files/size", post(modules::file::controller::file_controller::calc_size))
        .route("/api/files/download", get(modules::file::controller::file_controller::download_file))
        // RBAC me（任意登录用户可用）
        .route("/api/rbac/me", get(modules::sys::controller::user_controller::me))
        .route("/api/rbac/i18n/messages", get(modules::sys::controller::i18n_controller::get_i18n_messages))
        .route("/api/rbac/i18n", get(modules::sys::controller::i18n_controller::list_i18n))  // 读，全局可用
        .layer(from_fn_with_state(state.clone(), audit::middleware::audit_middleware)) // 先添加 = 后执行
        .layer(from_fn_with_state(state.clone(), middleware::auth_middleware));         // 后添加 = 先执行 = 注入 TokenInfo

    let admin_routes = Router::new()
        // 用户
        .route("/api/rbac/users", get(modules::sys::controller::user_controller::list_users).post(modules::sys::controller::user_controller::create_user))
        .route("/api/rbac/users/{id}", get(modules::sys::controller::user_controller::get_user).put(modules::sys::controller::user_controller::update_user).delete(modules::sys::controller::user_controller::delete_user))
        .route("/api/rbac/users/{id}/reset-password", post(modules::sys::controller::user_controller::reset_password))
        .route("/api/rbac/users/batch/reset-password", post(modules::sys::controller::user_controller::batch_reset_password))
        .route("/api/rbac/users/batch/disabled", post(modules::sys::controller::user_controller::batch_set_disabled))
        .route("/api/rbac/users/export", get(modules::sys::controller::user_controller::export_users))
        // 角色
        .route("/api/rbac/roles", get(modules::sys::controller::role_controller::list_roles).post(modules::sys::controller::role_controller::create_role))
        .route("/api/rbac/roles/{id}", put(modules::sys::controller::role_controller::update_role).delete(modules::sys::controller::role_controller::delete_role))
        .route("/api/rbac/roles/{id}/menus", get(modules::sys::controller::role_controller::get_role_menus).put(modules::sys::controller::role_controller::set_role_menus))
        // 部门
        .route("/api/rbac/depts", get(modules::sys::controller::dept_controller::list_depts).post(modules::sys::controller::dept_controller::create_dept))
        .route("/api/rbac/depts/tree", get(modules::sys::controller::dept_controller::dept_tree))
        .route("/api/rbac/depts/{id}", put(modules::sys::controller::dept_controller::update_dept).delete(modules::sys::controller::dept_controller::delete_dept))
        // 岗位
        .route("/api/rbac/posts", get(modules::sys::controller::post_controller::list_posts).post(modules::sys::controller::post_controller::create_post))
        .route("/api/rbac/posts/{id}", put(modules::sys::controller::post_controller::update_post).delete(modules::sys::controller::post_controller::delete_post))
        // 菜单
        .route("/api/rbac/menus", get(modules::sys::controller::menu_controller::list_menus).post(modules::sys::controller::menu_controller::create_menu))
        .route("/api/rbac/menus/batch-delete", post(modules::sys::controller::menu_controller::batch_delete_menus))
        .route("/api/rbac/menus/{id}", put(modules::sys::controller::menu_controller::update_menu).delete(modules::sys::controller::menu_controller::delete_menu))
        // 国际化
        .route("/api/rbac/i18n/locales", get(modules::sys::controller::i18n_controller::list_i18n_locales))
        .route("/api/rbac/i18n", post(modules::sys::controller::i18n_controller::upsert_i18n))  // 写，仅管理员
        .route("/api/rbac/i18n/{id}", delete(modules::sys::controller::i18n_controller::delete_i18n))
        // 字典
        .route("/api/rbac/dicts", get(modules::sys::controller::dict_controller::list_dicts).post(modules::sys::controller::dict_controller::create_dict))
        .route("/api/rbac/dicts/{id}", get(modules::sys::controller::dict_controller::get_dict).put(modules::sys::controller::dict_controller::update_dict).delete(modules::sys::controller::dict_controller::delete_dict))
        .route("/api/rbac/dicts/{dict_id}/items", post(modules::sys::controller::dict_controller::create_dict_item))
        .route("/api/rbac/dict-items/{id}", put(modules::sys::controller::dict_controller::update_dict_item).delete(modules::sys::controller::dict_controller::delete_dict_item))
        .layer(from_fn_with_state(state.clone(), audit::middleware::audit_middleware)) // 先添加 = 第3执行
        .layer(from_fn_with_state(state.clone(), middleware::auth_middleware))          // 第2添加 = 第2执行
        .layer(from_fn_with_state(state.clone(), middleware::require_admin));                       // 最后添加 = 第1执行

    // 静态文件服务（前端 SPA）
    let run_dir = crate::modules::common::config::get_run_dir();
    let static_dir = run_dir.join("static");
    tracing::info!("静态文件目录: {}", static_dir.display());

    let static_service = ServeDir::new(&static_dir)
        .not_found_service(ServeFile::new(static_dir.join("index.html")));

    public_routes
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(axum::middleware::from_fn(middleware::logging_middleware))              // 全局耗时
        .layer(CorsLayer::permissive())
        .fallback_service(static_service)
        .with_state(state)
}
