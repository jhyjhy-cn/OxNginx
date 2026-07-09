/// 从 User-Agent 提取浏览器名称
pub fn parse_browser(ua: &str) -> String {
    if ua.contains("Edg/") { return "Edge".into(); }
    if ua.contains("OPR/") || ua.contains("Opera") { return "Opera".into(); }
    if ua.contains("Firefox/") { return "Firefox".into(); }
    if ua.contains("Chrome/") && !ua.contains("Chromium") { return "Chrome".into(); }
    if ua.contains("Safari/") && !ua.contains("Chrome") { return "Safari".into(); }
    if ua.contains("Trident/") || ua.contains("MSIE") { return "IE".into(); }
    "Unknown".into()
}

/// 从 User-Agent 提取操作系统
pub fn parse_os(ua: &str) -> String {
    if ua.contains("Windows NT 10") { return "Windows 10/11".into(); }
    if ua.contains("Windows NT 6.3") { return "Windows 8.1".into(); }
    if ua.contains("Windows NT 6.1") { return "Windows 7".into(); }
    if ua.contains("Windows") { return "Windows".into(); }
    if ua.contains("Mac OS X") {
        // 提取版本号
        if let Some(start) = ua.find("Mac OS X ") {
            let ver = &ua[start + 9..];
            let end = ver.find(|c: char| c == ')' || c == ';').unwrap_or(ver.len());
            return format!("macOS {}", &ver[..end].replace('_', "."));
        }
        return "macOS".into();
    }
    if ua.contains("Android") {
        if let Some(start) = ua.find("Android ") {
            let ver = &ua[start + 8..];
            let end = ver.find(|c: char| c == ';' || c == ')').unwrap_or(ver.len());
            return format!("Android {}", &ver[..end]);
        }
        return "Android".into();
    }
    if ua.contains("iPhone") || ua.contains("iPad") { return "iOS".into(); }
    if ua.contains("Linux") { return "Linux".into(); }
    "Unknown".into()
}

/// 从请求头提取客户端 IP（支持 X-Forwarded-For / X-Real-IP）
pub fn extract_ip(headers: &axum::http::HeaderMap, addr: Option<std::net::SocketAddr>) -> String {
    // X-Forwarded-For 可能有多个 IP，取第一个
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = xff.split(',').next() {
            let ip = first.trim();
            if !ip.is_empty() { return ip.to_string(); }
        }
    }
    if let Some(xri) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        let ip = xri.trim();
        if !ip.is_empty() { return ip.to_string(); }
    }
    addr.map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".into())
}
