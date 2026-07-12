// site_backup_service 不需要 dao（纯文件 IO），从 src/service/site_backup_service.rs 原样迁移

use std::path::Path;

use serde::Serialize;

/// 备份文件信息
#[derive(Debug, Serialize)]
pub struct BackupFileInfo {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub created_at: String,
    pub remark: String,
}

/// 分页备份列表
#[derive(Debug, Serialize)]
pub struct BackupPage {
    pub items: Vec<BackupFileInfo>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

/// 获取站点备份目录
pub fn get_backup_dir(site_name: &str) -> anyhow::Result<std::path::PathBuf> {
    let base = crate::modules::common::config::get_run_dir();
    Ok(base.join("backup").join("sites").join(site_name))
}

/// 获取站点备份数量（用于站点列表）
pub fn count_backups(site_name: &str) -> u64 {
    let dir = match get_backup_dir(site_name) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    if !dir.exists() {
        return 0;
    }
    std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().ends_with(".zip"))
                .count() as u64
        })
        .unwrap_or(0)
}

/// 收集所有站点的备份数量
pub fn get_backup_counts(site_names: &[String]) -> std::collections::HashMap<String, u64> {
    let mut map = std::collections::HashMap::new();
    for name in site_names {
        map.insert(name.clone(), count_backups(name));
    }
    map
}

/// 列出站点备份（分页）
pub fn list_backups(site_name: &str, page: u32, page_size: u32) -> anyhow::Result<BackupPage> {
    let dir = get_backup_dir(site_name)?;
    if !dir.exists() {
        return Ok(BackupPage { items: vec![], total: 0, page, page_size });
    }

    let mut all = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if !metadata.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".zip") {
            continue;
        }
        let created = metadata
            .modified()
            .ok()
            .and_then(|t| {
                let dt: chrono::DateTime<chrono::Local> = t.into();
                Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
            })
            .unwrap_or_default();

        all.push(BackupFileInfo {
            filename: name,
            path: dir.to_string_lossy().to_string(),
            size: metadata.len(),
            created_at: created,
            remark: String::new(),
        });
    }

    all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let total = all.len() as u64;
    let start = ((page - 1) * page_size) as usize;
    let items = all.into_iter().skip(start).take(page_size as usize).collect();

    Ok(BackupPage { items, total, page, page_size })
}

/// 创建站点备份（压缩 root_path 到 zip）
pub fn create_backup(site_name: &str, root_path: &str) -> anyhow::Result<BackupFileInfo> {
    let dir = get_backup_dir(site_name)?;
    std::fs::create_dir_all(&dir)?;

    let now = chrono::Local::now();
    let ts = now.format("%Y%m%d_%H%M%S%3f").to_string();
    let filename = format!("{}_{}.zip", site_name, ts);
    let zip_path = dir.join(&filename);

    let root = Path::new(root_path);
    if !root.exists() {
        anyhow::bail!("站点根目录不存在: {}", root_path);
    }

    let file = std::fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, root, root, options)?;
    zip.finish()?;

    let metadata = std::fs::metadata(&zip_path)?;
    Ok(BackupFileInfo {
        filename,
        path: dir.to_string_lossy().to_string(),
        size: metadata.len(),
        created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        remark: String::new(),
    })
}

/// 递归添加目录到 zip
fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    current: &Path,
    options: zip::write::SimpleFileOptions,
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(base)
            .map_err(|e| anyhow::anyhow!("非法路径 {}: {}", path.display(), e))?
            .to_string_lossy()
            .replace('\\', "/");

        if path.is_dir() {
            zip.add_directory(format!("{}/", name), options)?;
            add_dir_to_zip(zip, base, &path, options)?;
        } else {
            zip.start_file(name, options)?;
            let mut f = std::fs::File::open(&path)?;
            std::io::copy(&mut f, zip)?;
        }
    }
    Ok(())
}

/// 获取备份文件路径
pub fn get_backup_path(site_name: &str, filename: &str) -> anyhow::Result<std::path::PathBuf> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        anyhow::bail!("非法文件名");
    }
    let path = get_backup_dir(site_name)?.join(filename);
    if !path.exists() {
        anyhow::bail!("备份文件不存在");
    }
    Ok(path)
}

/// 删除备份文件
pub fn delete_backup(site_name: &str, filename: &str) -> anyhow::Result<()> {
    let path = get_backup_path(site_name, filename)?;
    std::fs::remove_file(path)?;
    Ok(())
}