use anyhow::Result;
use chrono::DateTime;
use sqlx::Row;
use std::path::{Path, PathBuf};

use crate::modules::common::dto::file_dto::FileItem;
use crate::AppState;

/// 规范化路径，防止路径穿越
fn normalize_path(path: &str) -> Result<PathBuf> {
    let p = Path::new(path);
    let canonical = if p.exists() {
        std::fs::canonicalize(p)?
    } else {
        // 对于不存在的路径，做简单的规范化
        let mut components = Vec::new();
        for comp in p.components() {
            match comp {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {}
                other => components.push(other),
            }
        }
        components.iter().collect::<PathBuf>()
    };
    Ok(canonical)
}

/// 获取系统根目录列表（Windows 返回盘符，Linux 返回 ["/"]）
pub fn get_root_dirs() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        let mut drives = Vec::new();
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            if Path::new(&drive).exists() {
                drives.push(drive);
            }
        }
        drives
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec!["/".to_string()]
    }
}

/// 列出目录内容
pub async fn list_directory(_state: &AppState, path: &str) -> Result<Vec<FileItem>> {
    let dir_path = normalize_path(path)?;
    let mut entries = tokio::fs::read_dir(&dir_path).await?;

    let mut items = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue, // 跳过无权限项
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let full_path = entry.path().to_string_lossy().to_string();
        let is_dir = metadata.is_dir();

        let size = if is_dir { 0 } else { metadata.len() };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| {
                DateTime::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
                    0,
                )
            })
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();

        let permissions = get_permissions_string(&metadata, &full_path);
        let owner = get_owner_string(&full_path);
        let extension = if is_dir {
            String::new()
        } else {
            Path::new(&name)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        items.push(FileItem {
            name,
            path: full_path,
            is_dir,
            size,
            permissions,
            owner,
            modified,
            extension,
            note: None,
        });
    }

    // 目录排前面，文件排后面，同类按名称排序
    items.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(items)
}

/// 批量查询文件备注
pub async fn get_notes(state: &AppState, paths: &[String]) -> Result<Vec<(String, String)>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    // 构建 IN 查询
    let placeholders: Vec<&str> = paths.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT path, note FROM file_notes WHERE path IN ({})",
        placeholders.join(",")
    );
    let mut query = sqlx::query(&sql);
    for p in paths {
        query = query.bind(p);
    }
    let rows = query.fetch_all(state.db.pool()).await?;
    let mut notes = Vec::new();
    for row in rows {
        let path: String = row.get("path");
        let note: String = row.get("note");
        notes.push((path, note));
    }
    Ok(notes)
}

/// 保存文件备注
pub async fn save_note(state: &AppState, path: &str, note: &str) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO file_notes (path, note) VALUES (?, ?)")
        .bind(path)
        .bind(note)
        .execute(state.db.pool())
        .await?;
    Ok(())
}

/// 删除文件备注
pub async fn delete_note(state: &AppState, path: &str) -> Result<()> {
    sqlx::query("DELETE FROM file_notes WHERE path = ?")
        .bind(path)
        .execute(state.db.pool())
        .await?;
    Ok(())
}

/// 读取文件内容（文本）
pub async fn read_file(path: &str) -> Result<(bool, String)> {
    let bytes = tokio::fs::read(path).await?;
    match String::from_utf8(bytes) {
        Ok(content) => Ok((true, content)),
        Err(_) => Ok((false, String::new())),
    }
}

/// 写入文件内容
pub async fn write_file(path: &str, content: &str) -> Result<()> {
    // 确保父目录存在
    if let Some(parent) = Path::new(path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(path, content).await?;
    Ok(())
}

/// 创建目录
pub async fn create_dir(parent: &str, name: &str) -> Result<String> {
    let path = Path::new(parent).join(name);
    tokio::fs::create_dir_all(&path).await?;
    Ok(path.to_string_lossy().to_string())
}

/// 创建空文件
pub async fn create_file(parent: &str, name: &str) -> Result<String> {
    let path = Path::new(parent).join(name);
    tokio::fs::write(&path, "").await?;
    Ok(path.to_string_lossy().to_string())
}

/// 重命名
pub async fn rename(path: &str, new_name: &str) -> Result<String> {
    let old = Path::new(path);
    let parent = old.parent().ok_or_else(|| anyhow::anyhow!("无法获取父目录"))?;
    let new_path = parent.join(new_name);
    tokio::fs::rename(old, &new_path).await?;
    Ok(new_path.to_string_lossy().to_string())
}

/// 移动文件/目录
pub async fn move_path(source: &str, destination: &str) -> Result<()> {
    tokio::fs::rename(source, destination).await?;
    Ok(())
}

/// 复制文件/目录
pub async fn copy_path(source: &str, destination: &str) -> Result<()> {
    let src = Path::new(source);
    if src.is_dir() {
        copy_dir_recursive(src, Path::new(destination)).await?;
    } else {
        tokio::fs::copy(source, destination).await?;
    }
    Ok(())
}

/// 递归复制目录
async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    tokio::fs::create_dir_all(dst).await?;
    let mut entries = tokio::fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await?;
        }
    }
    Ok(())
}

/// 删除文件/目录
pub async fn delete_path(path: &str) -> Result<()> {
    let p = Path::new(path);
    if p.is_dir() {
        tokio::fs::remove_dir_all(p).await?;
    } else {
        tokio::fs::remove_file(p).await?;
    }
    Ok(())
}

/// 修改文件权限（仅 Linux/macOS）
#[cfg(unix)]
pub async fn chmod(path: &str, mode: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode_val = u32::from_str_radix(mode, 8)?;
    let perm = std::fs::Permissions::from_mode(mode_val);
    tokio::fs::set_permissions(path, perm).await?;
    Ok(())
}

#[cfg(not(unix))]
pub async fn chmod(_path: &str, _mode: &str) -> Result<()> {
    Err(anyhow::anyhow!("修改权限仅支持 Linux/macOS 系统"))
}

/// 压缩文件
pub async fn compress(paths: &[String], destination: &str, format: &str) -> Result<()> {
    let paths = paths.to_vec();
    let dest = destination.to_string();
    let fmt = format.to_string();

    tokio::task::spawn_blocking(move || -> Result<()> {
        match fmt.as_str() {
            "zip" => compress_zip(&paths, &dest),
            "tar.gz" | "tgz" => compress_tar_gz(&paths, &dest),
            _ => Err(anyhow::anyhow!("不支持的压缩格式: {}", fmt)),
        }
    })
    .await?
}

fn compress_zip(paths: &[String], destination: &str) -> Result<()> {
    let file = std::fs::File::create(destination)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_file() {
            let name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("无效文件名"))?
                .to_string_lossy();
            zip.start_file(name, options)?;
            let content = std::fs::read(path)?;
            std::io::Write::write_all(&mut zip, &content)?;
        } else if path.is_dir() {
            add_dir_to_zip(&mut zip, path, path, options)?;
        }
    }

    zip.finish()?;
    Ok(())
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    dir: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(base)
            .unwrap()
            .to_string_lossy()
            .to_string();
        if path.is_file() {
            zip.start_file(&name, options)?;
            let content = std::fs::read(&path)?;
            std::io::Write::write_all(zip, &content)?;
        } else if path.is_dir() {
            zip.add_directory(&name, options)?;
            add_dir_to_zip(zip, base, &path, options)?;
        }
    }
    Ok(())
}

fn compress_tar_gz(paths: &[String], destination: &str) -> Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let file = std::fs::File::create(destination)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_file() {
            let name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("无效文件名"))?
                .to_string_lossy()
                .to_string();
            tar.append_path_with_name(path, &name)?;
        } else if path.is_dir() {
            tar.append_dir_all(
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                path,
            )?;
        }
    }

    tar.into_inner()?.finish()?;
    Ok(())
}

/// 解压文件
pub async fn extract(path: &str, destination: &str) -> Result<()> {
    let src = path.to_string();
    let dst = destination.to_string();

    // 根据扩展名判断格式
    let lower = src.to_lowercase();
    if lower.ends_with(".zip") {
        tokio::task::spawn_blocking(move || extract_zip(&src, &dst)).await??;
    } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        tokio::task::spawn_blocking(move || extract_tar_gz(&src, &dst)).await??;
    } else {
        return Err(anyhow::anyhow!("不支持的压缩格式，请使用 .zip 或 .tar.gz"));
    }

    Ok(())
}

fn extract_zip(source: &str, destination: &str) -> Result<()> {
    let file = std::fs::File::open(source)?;
    let mut archive = zip::ZipArchive::new(file)?;
    // zip crate 0.6+ 使用 extract 方法
    archive.extract(destination)?;
    Ok(())
}

fn extract_tar_gz(source: &str, destination: &str) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file = std::fs::File::open(source)?;
    let dec = GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    archive.unpack(destination)?;
    Ok(())
}

/// 获取权限字符串
fn get_permissions_string(metadata: &std::fs::Metadata, _path: &str) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        let perms = format!(
            "{}{:o}{:o}{:o}",
            if metadata.is_dir() { 'd' } else { '-' },
            (mode >> 6) & 7,
            (mode >> 3) & 7,
            mode & 7,
        );
        // 转换为 rwx 格式
        let perm_chars: String = perms
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    return c;
                }
                match c {
                    '0' => "---",
                    '1' => "--x",
                    '2' => "-w-",
                    '3' => "-wx",
                    '4' => "r--",
                    '5' => "r-x",
                    '6' => "rw-",
                    '7' => "rwx",
                    _ => "???",
                }
                .to_string()
            })
            .collect::<Vec<_>>()
            .join("");
        // 修正：第一个字符后每3个一组
        let first = &perms[0..1];
        let rest: String = perms[1..]
            .chars()
            .map(|c| match c {
                '0' => "---",
                '1' => "--x",
                '2' => "-w-",
                '3' => "-wx",
                '4' => "r--",
                '5' => "r-x",
                '6' => "rw-",
                '7' => "rwx",
                _ => "???",
            })
            .collect::<String>();
        format!("{}{}", first, rest)
    }
    #[cfg(windows)]
    {
        let mut attrs = String::new();
        if metadata.permissions().readonly() {
            attrs.push_str("R");
        }
        // Windows 没有 Unix 风格权限，显示文件属性
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            let attrs_raw = metadata.file_attributes();
            if attrs_raw & 0x2 != 0 {
                attrs.push_str("H"); // Hidden
            }
            if attrs_raw & 0x4 != 0 {
                attrs.push_str("S"); // System
            }
            if attrs_raw & 0x100 != 0 {
                attrs.push_str("A"); // Archive
            }
        }
        if attrs.is_empty() {
            attrs.push_str("A"); // 普通文件默认 Archive
        }
        if metadata.is_dir() {
            format!("D({})", attrs)
        } else {
            format!("F({})", attrs)
        }
    }
}

/// 获取文件所有者
fn get_owner_string(_path: &str) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(_path) {
            let uid = metadata.uid();
            let gid = metadata.gid();
            let user = get_username(uid).unwrap_or_else(|| uid.to_string());
            let group = get_groupname(gid).unwrap_or_else(|| gid.to_string());
            return format!("{}:{}", user, group);
        }
        String::new()
    }
    #[cfg(not(unix))]
    {
        // Windows 下简化显示
        "SYSTEM".to_string()
    }
}

/// 获取用户名（Linux/macOS）
#[cfg(unix)]
fn get_username(uid: u32) -> Option<String> {
    // 使用 uzers crate 解析
    uzers::get_user_by_uid(uid).map(|u| u.name().to_string_lossy().to_string())
}

/// 获取组名（Linux/macOS）
#[cfg(unix)]
fn get_groupname(gid: u32) -> Option<String> {
    uzers::get_group_by_gid(gid).map(|g| g.name().to_string_lossy().to_string())
}

/// 需要跳过的目录名（系统保护或体积巨大的目录）
fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        "System Volume Information"
            | "$Recycle.Bin"
            | "$RECYCLE.BIN"
            | "Recovery"
            | "PerfLogs"
    )
}

/// 同步计算文件或目录大小（跳过无权限项和系统目录）
fn calc_size_sync(path: &Path) -> u64 {
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }
    if !path.is_dir() {
        return 0;
    }
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_file() {
            total += meta.len();
        } else if meta.is_dir() && !should_skip_dir(&name) {
            total += calc_size_sync(&entry.path());
        }
    }
    total
}

/// 计算文件或目录大小（spawn_blocking 避免异步开销）
pub async fn calc_size(path: &str) -> Result<u64> {
    let p = path.to_string();
    let size = tokio::task::spawn_blocking(move || calc_size_sync(Path::new(&p)))
        .await
        .unwrap_or(0);
    Ok(size)
}
