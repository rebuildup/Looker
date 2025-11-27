use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// スキャン結果
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
    #[allow(dead_code)]
    pub size: u64,
    pub modified: DateTime<Local>,
    pub is_dir: bool,
}

/// ディレクトリを走査するシンプルなラッパー
pub struct DriveScanner;

impl DriveScanner {
    /// 指定パス以下を再帰的に列挙
    pub fn scan(path: &Path) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let metadata = entry
                .metadata()
                .with_context(|| format!("メタデータ取得に失敗: {:?}", entry.path()))?;

            let modified = metadata
                .modified()
                .map(DateTime::<Local>::from)
                .unwrap_or_else(|_| Local::now());

            let path = entry.path().to_path_buf();
            let name = entry.file_name().to_string_lossy().to_string();
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            files.push(FileInfo {
                path,
                name,
                extension,
                size: metadata.len(),
                modified,
                is_dir: metadata.is_dir(),
            });
        }

        Ok(files)
    }
}
