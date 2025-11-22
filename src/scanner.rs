use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use walkdir::WalkDir;
use chrono::{DateTime, Local};

/// ファイル情報
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

/// ドライブ全体をスキャンする
pub struct DriveScanner;

impl DriveScanner {
    /// 指定されたパスを再帰的にスキャン
    pub fn scan(path: &Path) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let metadata = entry.metadata()
                .with_context(|| format!("メタデータの取得に失敗: {:?}", entry.path()))?;

            let modified = metadata
                .modified()
                .map(|t| {
                    DateTime::<Local>::from(t)
                })
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

    /// ファイルをカテゴリ別に分類
    pub fn categorize_files(files: &[FileInfo]) -> FileCategories {
        let mut categories = FileCategories::default();

        for file in files {
            if file.is_dir {
                continue;
            }

            match file.extension.as_str() {
                // 画像ファイル
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => {
                    categories.images.push(file.clone());
                }
                // 動画ファイル
                "mp4" | "avi" | "mov" | "mkv" | "wmv" | "flv" | "webm" | "m4v" => {
                    categories.videos.push(file.clone());
                }
                // 音声ファイル
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => {
                    categories.audio.push(file.clone());
                }
                // ドキュメントファイル
                "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" => {
                    categories.documents.push(file.clone());
                }
                // アーカイブファイル
                "zip" | "rar" | "7z" | "tar" | "gz" => {
                    categories.archives.push(file.clone());
                }
                // その他
                _ => {
                    categories.others.push(file.clone());
                }
            }
        }

        categories
    }
}

#[derive(Debug, Default)]
pub struct FileCategories {
    pub images: Vec<FileInfo>,
    pub videos: Vec<FileInfo>,
    pub audio: Vec<FileInfo>,
    pub documents: Vec<FileInfo>,
    pub archives: Vec<FileInfo>,
    pub others: Vec<FileInfo>,
}

impl FileCategories {
    pub fn total_count(&self) -> usize {
        self.images.len()
            + self.videos.len()
            + self.audio.len()
            + self.documents.len()
            + self.archives.len()
            + self.others.len()
    }

    pub fn print_summary(&self) {
        println!("=== ファイル分類結果 ===");
        println!("画像ファイル: {}個", self.images.len());
        println!("動画ファイル: {}個", self.videos.len());
        println!("音声ファイル: {}個", self.audio.len());
        println!("ドキュメント: {}個", self.documents.len());
        println!("アーカイブ: {}個", self.archives.len());
        println!("その他: {}個", self.others.len());
        println!("合計: {}個", self.total_count());
    }
}

