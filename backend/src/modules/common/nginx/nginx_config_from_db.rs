use crate::AppState;
use crate::modules::sys::service::param_service::NginxConfigFromDb;

/// 从 AppState 获取 Nginx 配置（从数据库 sys_params 读取）
pub async fn get_nginx_config(state: &AppState) -> anyhow::Result<NginxConfigFromDb> {
    crate::modules::sys::service::param_service::get_nginx_config(state.db.pool()).await
}
