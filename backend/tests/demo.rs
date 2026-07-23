//! backend/tests/ 集成测试 —— 写法示范
//!
//! ponytail: 这是"写法示范"，不测项目内部逻辑。
//! 原因：ox-nginx 是 binary crate（只有 main.rs，无 lib.rs），
//! tests/ 作为独立 crate 无法 use 到 modules/ 里的内部函数。
//! 升级路径：要测项目真实逻辑，要么把测试写进源文件的 #[cfg(test)]，
//! 要么给 backend 加 lib.rs 导出待测纯函数，再来这里测。
//!
//! 运行：cargo test --test demo

/// 示例纯函数。真实项目里这里会是你导出的业务函数。
fn build_server_block(host: &str, port: u16) -> String {
    format!("server {{ listen {}; server_name {}; }}", port, host)
}

/// 1. 同步测试 + assert_eq!：最基本的形态。
#[test]
fn test_build_server_block() {
    assert_eq!(
        build_server_block("a.com", 80),
        "server { listen 80; server_name a.com; }"
    );
}

/// 2. 返回 Result 的测试：可以用 ? 串起来，失败自动报错，无需 unwrap。
#[test]
fn test_with_result() -> Result<(), String> {
    let out = build_server_block("b.com", 443);
    if out.contains("server_name b.com") {
        Ok(())
    } else {
        Err(format!("缺少 server_name: {out}"))
    }
}

/// 3. 异步测试：#[tokio::test] 起一个 runtime。
///    tokio 在主 crate [dependencies] 里（feature "full" 含 macros），
///    tests/ 可直接用，无需额外加 dev-dependency。
async fn name_len(s: &str) -> usize {
    s.len()
}

#[tokio::test]
async fn test_async() {
    assert_eq!(name_len("nginx").await, 5);
}
