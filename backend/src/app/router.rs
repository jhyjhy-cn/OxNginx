use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::api;
use crate::middleware;
use super::state::AppState;

/// 构建应用路由
pub fn build(state: AppState) -> Router {
    // 公开路由（无需认证）
    let public_routes = Router::new()
        .route("/api/login", post(api::auth_api::login))
        .route("/api/logout", post(api::auth_api::logout))
        .route("/api/setup", post(api::auth_api::setup))
        .route("/api/setup/status", get(api::auth_api::setup_status))
        .route("/api/auth/public-key", get(api::auth_api::get_public_key))
        .route("/api/terminal/ws", get(api::terminal_api::terminal_ws))
        .route("/api/dashboard/ws", get(api::dashboard_ws::dashboard_ws));

    // 需要认证的路由
    let protected_routes = Router::new()
        .route("/api/dashboard", get(api::dashboard_api::get_dashboard))
        .route("/api/sites", get(api::site_api::list_sites))
        .route("/api/sites/with-certs", get(api::site_api::list_sites_with_certs))
        .route("/api/sites", post(api::site_api::create_site))
        .route("/api/sites/:id", get(api::site_api::get_site))
        .route("/api/sites/:id", put(api::site_api::update_site))
        .route("/api/sites/:id", delete(api::site_api::delete_site))
        .route("/api/sites/:id/deploy-ssl", post(api::site_api::deploy_ssl))
        .route("/api/sites/batch/enable", post(api::site_api::batch_enable))
        .route("/api/sites/batch/disable", post(api::site_api::batch_disable))
        .route("/api/sites/batch/delete", post(api::site_api::batch_delete))
        // 反向代理
        .route("/api/sites/:id/proxies", get(api::reverse_proxy_api::list_proxies))
        .route("/api/sites/:id/proxies", post(api::reverse_proxy_api::create_proxy))
        .route("/api/proxies/:id", put(api::reverse_proxy_api::update_proxy))
        .route("/api/proxies/:id", delete(api::reverse_proxy_api::delete_proxy))
        .route("/api/certificates", get(api::auth_api::list_certificates))
        .route("/api/certificate/apply", post(api::auth_api::apply_certificate))
        .route("/api/certificate/renew", post(api::auth_api::renew_certificate))
        .route("/api/change-password", post(api::auth_api::change_password))
        .route("/api/change-username", post(api::auth_api::change_username))
        .route("/api/nginx/test", post(api::nginx_api::test_config))
        .route("/api/nginx/reload", post(api::nginx_api::reload))
        .route("/api/nginx/status", get(api::nginx_api::status))
        .route("/api/nginx/start", post(api::nginx_api::start))
        .route("/api/nginx/stop", post(api::nginx_api::stop))
        .route("/api/nginx/restart", post(api::nginx_api::restart))
        .route("/api/nginx/install", post(api::nginx_api::install))
        .route("/api/log/access", get(api::log_api::access_log))
        .route("/api/log/error", get(api::log_api::error_log))
        .route("/api/backups/:id", get(api::backup_api::list_backups))
        .route("/api/backups/:id", post(api::backup_api::create_backup))
        .route("/api/backups/:id", delete(api::backup_api::delete_backup))
        .route("/api/backups/restore/:id", post(api::backup_api::restore_backup))
        .route("/api/backups/diff", post(api::backup_api::diff_backups))
        .route("/api/sites/:id/backups", get(api::site_backup_api::list_site_backups))
        .route("/api/sites/:id/backups", post(api::site_backup_api::create_site_backup))
        .route("/api/sites/:id/backups/:filename", delete(api::site_backup_api::delete_site_backup))
        .route("/api/sites/:id/backups/:filename/download", get(api::site_backup_api::download_site_backup))
        .route("/api/sites/:id/backups/batch-delete", post(api::site_backup_api::batch_delete_site_backups))
        .route("/api/config/main", get(api::config_api::get_main_config))
        .route("/api/config/main", put(api::config_api::save_main_config))
        .route("/api/config/files", get(api::config_api::list_config_files))
        .route("/api/config/file/:name", get(api::config_api::get_site_config))
        .route("/api/config/file/:name", put(api::config_api::save_site_config))
        .route("/api/config/file/:name/toggle", post(api::config_api::toggle_site_config))
        .route("/api/config/file/:name", delete(api::config_api::delete_site_config))
        .route("/api/upstreams", get(api::upstream_api::list_upstreams))
        .route("/api/upstreams", post(api::upstream_api::create_upstream))
        .route("/api/upstreams/:id", get(api::upstream_api::get_upstream))
        .route("/api/upstreams/:id", put(api::upstream_api::update_upstream))
        .route("/api/upstreams/:id", delete(api::upstream_api::delete_upstream))
        .route("/api/access-rules", get(api::access_api::list_rules))
        .route("/api/access-rules", post(api::access_api::create_rule))
        .route("/api/access-rules/:id", get(api::access_api::get_rule))
        .route("/api/access-rules/:id", put(api::access_api::update_rule))
        .route("/api/access-rules/:id", delete(api::access_api::delete_rule))
        .route("/api/templates", get(api::template_api::list_templates))
        .route("/api/templates", post(api::template_api::create_template))
        .route("/api/templates/:id", get(api::template_api::get_template))
        .route("/api/templates/:id", put(api::template_api::update_template))
        .route("/api/templates/:id", delete(api::template_api::delete_template))
        .route("/api/templates/:id/preview", post(api::template_api::preview_template))
        .route("/api/settings", get(api::settings_api::get_settings))
        .route("/api/settings", put(api::settings_api::update_settings))
        .route("/api/files/list", post(api::file_api::list_files))
        .route("/api/files/roots", get(api::file_api::list_roots))
        .route("/api/files/read", post(api::file_api::read_file))
        .route("/api/files/write", post(api::file_api::write_file))
        .route("/api/files/mkdir", post(api::file_api::mkdir))
        .route("/api/files/touch", post(api::file_api::touch))
        .route("/api/files/rename", post(api::file_api::rename))
        .route("/api/files/move", post(api::file_api::move_file))
        .route("/api/files/copy", post(api::file_api::copy_file))
        .route("/api/files/delete", delete(api::file_api::delete_file))
        .route("/api/files/chmod", post(api::file_api::chmod))
        .route("/api/files/compress", post(api::file_api::compress))
        .route("/api/files/extract", post(api::file_api::extract))
        .route("/api/files/note", post(api::file_api::save_note))
        .route("/api/files/size", post(api::file_api::calc_size))
        .route("/api/files/download", get(api::file_api::download_file))
        // RBAC me（任意登录用户可用）
        .route("/api/rbac/me", get(api::rbac_api::me))
        .route("/api/rbac/i18n/messages", get(api::rbac_api::get_i18n_messages))
        .layer(from_fn_with_state(state.clone(), middleware::auth_middleware));

    // 管理员路由（需 super_admin / username=='admin'）
    let admin_routes = Router::new()
        .route("/api/rbac/users", get(api::rbac_api::list_users).post(api::rbac_api::create_user))
        .route("/api/rbac/users/:id", put(api::rbac_api::update_user).delete(api::rbac_api::delete_user))
        .route("/api/rbac/users/:id/reset-password", post(api::rbac_api::reset_password))
        .route("/api/rbac/roles", get(api::rbac_api::list_roles).post(api::rbac_api::create_role))
        .route("/api/rbac/roles/:id", put(api::rbac_api::update_role).delete(api::rbac_api::delete_role))
        .route("/api/rbac/roles/:id/menus", put(api::rbac_api::set_role_menus))
        .route("/api/rbac/depts", get(api::rbac_api::list_depts).post(api::rbac_api::create_dept))
        .route("/api/rbac/depts/:id", put(api::rbac_api::update_dept).delete(api::rbac_api::delete_dept))
        .route("/api/rbac/posts", get(api::rbac_api::list_posts).post(api::rbac_api::create_post))
        .route("/api/rbac/posts/:id", put(api::rbac_api::update_post).delete(api::rbac_api::delete_post))
        .route("/api/rbac/menus", get(api::rbac_api::list_menus).post(api::rbac_api::create_menu))
        .route("/api/rbac/menus/batch-delete", post(api::rbac_api::batch_delete_menus))
        .route("/api/rbac/menus/:id", put(api::rbac_api::update_menu).delete(api::rbac_api::delete_menu))
        // 国际化
        .route("/api/rbac/i18n/locales", get(api::rbac_api::list_i18n_locales))
        .route("/api/rbac/i18n", get(api::rbac_api::list_i18n).post(api::rbac_api::upsert_i18n))
        .route("/api/rbac/i18n/:id", delete(api::rbac_api::delete_i18n))
        // 字典
        .route("/api/rbac/dicts", get(api::rbac_api::list_dicts).post(api::rbac_api::create_dict))
        .route("/api/rbac/dicts/:id", get(api::rbac_api::get_dict).put(api::rbac_api::update_dict).delete(api::rbac_api::delete_dict))
        .route("/api/rbac/dicts/:dict_id/items", post(api::rbac_api::create_dict_item))
        .route("/api/rbac/dict-items/:id", put(api::rbac_api::update_dict_item).delete(api::rbac_api::delete_dict_item))
        .layer(from_fn_with_state(state.clone(), middleware::require_admin))
        .layer(from_fn_with_state(state.clone(), middleware::auth_middleware));

    // 静态文件服务（前端 SPA）
    // 使用 exe 所在目录下的 static 目录
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let static_dir = exe_dir.join("static");
    tracing::info!("静态文件目录: {}", static_dir.display());

    let static_service = ServeDir::new(&static_dir)
        .not_found_service(ServeFile::new(static_dir.join("index.html")));

    public_routes
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(axum::middleware::from_fn(middleware::logging_middleware))
        .layer(CorsLayer::permissive())
        .fallback_service(static_service)
        .with_state(state)
}
