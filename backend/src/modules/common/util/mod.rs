pub mod cert_service;
pub mod cmd;
pub mod datetime;
pub mod excel;
pub mod ua_parser;

/// 读取日志文件最后N行
pub async fn read_log_tail(log_path: &str, lines: usize) -> anyhow::Result<Vec<String>> {
    let content = tokio::fs::read_to_string(log_path).await?;
    let all_lines: Vec<&str> = content.lines().collect();
    let start = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };

    Ok(all_lines[start..].iter().map(|s| s.to_string()).collect())
}
