use std::fs;

fn main() {
    // 自动创建 bundle 资源目录，避免构建报错
    for dir in ["server", "backup", "ssl", "wwwlogs", "wwwroot"] {
        fs::create_dir_all(format!("bundle/{dir}")).unwrap();
    }

    tauri_build::build()
}
