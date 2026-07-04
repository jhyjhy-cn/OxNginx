use serde::{Deserialize, Serialize};

/// 文件列表项
#[derive(Debug, Serialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
    pub owner: String,
    pub modified: String,
    pub extension: String,
    pub note: Option<String>,
}

/// 文件列表响应
#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub path: String,
    pub parent: Option<String>,
    pub items: Vec<FileItem>,
    pub total: usize,
    pub dir_count: usize,
    pub file_count: usize,
}

/// 写入文件请求
#[derive(Debug, Deserialize)]
pub struct FileWriteRequest {
    pub path: String,
    pub content: String,
}

/// 创建目录请求
#[derive(Debug, Deserialize)]
pub struct FileMkdirRequest {
    pub path: String,
    pub name: String,
}

/// 创建文件请求
#[derive(Debug, Deserialize)]
pub struct FileTouchRequest {
    pub path: String,
    pub name: String,
}

/// 重命名请求
#[derive(Debug, Deserialize)]
pub struct FileRenameRequest {
    pub path: String,
    pub new_name: String,
}

/// 移动请求
#[derive(Debug, Deserialize)]
pub struct FileMoveRequest {
    pub source: String,
    pub destination: String,
}

/// 复制请求
#[derive(Debug, Deserialize)]
pub struct FileCopyRequest {
    pub source: String,
    pub destination: String,
}

/// 删除请求
#[derive(Debug, Deserialize)]
pub struct FileDeleteRequest {
    pub path: String,
}

/// 修改权限请求（仅 Linux 生效）
#[derive(Debug, Deserialize)]
pub struct FileChmodRequest {
    pub path: String,
    pub mode: String,
}

/// 压缩请求
#[derive(Debug, Deserialize)]
pub struct FileCompressRequest {
    pub paths: Vec<String>,
    pub destination: String,
    pub format: String,
}

/// 解压请求
#[derive(Debug, Deserialize)]
pub struct FileExtractRequest {
    pub path: String,
    pub destination: String,
}

/// 保存备注请求
#[derive(Debug, Deserialize)]
pub struct FileNoteRequest {
    pub path: String,
    pub note: String,
}
