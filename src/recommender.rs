use std::path::PathBuf;
use crate::scanner::FileInfo;
use crate::structure::PathType;

/// ファイルの推奨配置を提案
pub struct PlacementRecommender;

impl PlacementRecommender {
    /// ファイルの推奨配置先を決定
    pub fn recommend_placement(
        file: &FileInfo,
        root: &PathBuf,
    ) -> Recommendation {
        let extension = &file.extension;

        // 拡張子に基づいて分類
        match extension.as_str() {
            // 動画ファイル → 2_assets/footage
            "mp4" | "avi" | "mov" | "mkv" | "wmv" | "flv" | "webm" | "m4v" => {
                Recommendation::new(
                    root.join("2_assets/footage"),
                    PathType::AssetsFootage,
                    "動画ファイルはfootageフォルダに配置することを推奨します",
                )
            }
            // 画像ファイル → 2_assets配下（種類によって分類）
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => {
                // パスに基づいてより詳細な分類
                let path_str = file.path.to_string_lossy().to_lowercase();
                if path_str.contains("illust") || path_str.contains("illustration") {
                    Recommendation::new(
                        root.join("2_assets/illust"),
                        PathType::AssetsIllust,
                        "イラストファイルはillustフォルダに配置することを推奨します",
                    )
                } else if path_str.contains("graphic") || path_str.contains("design") {
                    Recommendation::new(
                        root.join("2_assets/graphic"),
                        PathType::AssetsGraphic,
                        "グラフィックファイルはgraphicフォルダに配置することを推奨します",
                    )
                } else {
                    Recommendation::new(
                        root.join("2_assets/photo"),
                        PathType::AssetsPhoto,
                        "写真ファイルはphotoフォルダに配置することを推奨します",
                    )
                }
            }
            // 音声ファイル → 2_assets/bgm または sfx
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => {
                let path_str = file.path.to_string_lossy().to_lowercase();
                if path_str.contains("bgm") || path_str.contains("music") {
                    Recommendation::new(
                        root.join("2_assets/bgm"),
                        PathType::AssetsBgm,
                        "BGMファイルはbgmフォルダに配置することを推奨します",
                    )
                } else {
                    Recommendation::new(
                        root.join("2_assets/sfx"),
                        PathType::AssetsSfx,
                        "効果音ファイルはsfxフォルダに配置することを推奨します",
                    )
                }
            }
            // ドキュメントファイル → 3_docs
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" => {
                Recommendation::new(
                    root.join("3_docs"),
                    PathType::Docs,
                    "ドキュメントファイルは3_docsフォルダに配置することを推奨します",
                )
            }
            // アーカイブファイル → 9_archive
            "zip" | "rar" | "7z" | "tar" | "gz" => {
                Recommendation::new(
                    root.join("9_archive"),
                    PathType::Archive,
                    "アーカイブファイルは9_archiveフォルダに配置することを推奨します",
                )
            }
            // その他 → 0_inbox
            _ => {
                Recommendation::new(
                    root.join("0_inbox"),
                    PathType::Inbox,
                    "未分類ファイルは0_inboxフォルダに配置することを推奨します",
                )
            }
        }
    }

    /// 複数のファイルに対する推奨配置を生成
    pub fn recommend_batch(
        files: &[FileInfo],
        root: &PathBuf,
    ) -> Vec<FileRecommendation> {
        files
            .iter()
            .map(|file| {
                let recommendation = Self::recommend_placement(file, root);
                FileRecommendation {
                    file: file.clone(),
                    recommendation,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub target_path: PathBuf,
    #[allow(dead_code)]
    pub path_type: PathType,
    pub reason: String,
}

impl Recommendation {
    pub fn new(target_path: PathBuf, path_type: PathType, reason: &str) -> Self {
        Self {
            target_path,
            path_type,
            reason: reason.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileRecommendation {
    pub file: FileInfo,
    pub recommendation: Recommendation,
}

impl FileRecommendation {
    pub fn print(&self) {
        println!("ファイル: {:?}", self.file.path);
        println!("  推奨先: {:?}", self.recommendation.target_path);
        println!("  理由: {}", self.recommendation.reason);
        println!();
    }
}

