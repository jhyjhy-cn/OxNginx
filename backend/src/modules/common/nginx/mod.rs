// 拆文件后 mod.rs 只做模块声明 + 显式 re-export
// 子文件加新 pub 时不会自动导出到 `crate::modules::common::nginx::*`

mod common_directives;
mod ensure_include;
mod nginx_config_from_db;
mod process;
mod site_config;
mod upstream;

// site_config.rs
pub use site_config::{
    create_default_index, generate_site_config, generate_site_config_with_proxies,
    remove_site_config, write_site_config,
};

// upstream.rs
pub use upstream::{generate_upstream_config, generate_upstream_config_from_request};

// process.rs
pub use process::{
    get_nginx_status, install_nginx, reload_nginx, restart_nginx, start_nginx, stop_nginx,
    test_config, NginxInstallResult,
};

// ensure_include.rs
pub use ensure_include::ensure_sites_enabled_include;

// nginx_config_from_db.rs
pub use nginx_config_from_db::get_nginx_config;
