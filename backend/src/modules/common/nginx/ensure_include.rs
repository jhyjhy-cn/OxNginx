/// 确保 nginx.conf 中包含 sites-enabled 目录的 include 指令
pub async fn ensure_sites_enabled_include(nginx_config: &str, sites_enabled: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        use crate::modules::common::util::cmd::silent_tokio_command;

        let content = tokio::fs::read_to_string(nginx_config).await?;
        if content.contains("sites-enabled") {
            return Ok(());
        }
        let sites_path = sites_enabled.replace('\\', "/");
        let include_line = format!("\n    include {}/*.conf;\n", sites_path);
        if let Some(pos) = content.rfind('}') {
            let mut new_content = content[..pos].to_string();
            new_content.push_str(&include_line);
            new_content.push_str("}\n");
            let tmp = "/tmp/.oxnginx_nginx_conf_tmp";
            tokio::fs::write(tmp, &new_content).await?;
            let _ = silent_tokio_command("mv")
                .args([tmp, nginx_config])
                .output()
                .await;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let content = tokio::fs::read_to_string(nginx_config).await?;
        if content.contains("sites-enabled") {
            return Ok(());
        }
        let sites_path = sites_enabled.replace('\\', "/");
        let include_line = format!("\n    include {}/*.conf;\n", sites_path);
        if let Some(pos) = content.rfind('}') {
            let mut new_content = content[..pos].to_string();
            new_content.push_str(&include_line);
            new_content.push_str("}\n");
            tokio::fs::write(nginx_config, new_content).await?;
        }
        Ok(())
    }
}
