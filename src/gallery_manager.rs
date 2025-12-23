use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use walkdir::WalkDir;

use crate::ui::UI;

/// 1_projects以下のプロジェクト成果物を5_galleryにショートカット作成
pub struct GalleryManager;

impl GalleryManager {
    /// プロジェクトフォルダを探索してギャラリーショートカットを作成
    pub fn create_shortcuts(root: &Path) -> Result<()> {
        let projects_dir = root.join("1_projects");
        let gallery_dir = root.join("5_gallery");

        if !projects_dir.exists() {
            return Err(anyhow!("1_projects フォルダが見つかりません: {}", projects_dir.display()));
        }

        // 5_gallery がなければ作成
        if !gallery_dir.exists() {
            fs::create_dir_all(&gallery_dir)
                .with_context(|| format!("5_gallery の作成に失敗: {}", gallery_dir.display()))?;
            UI::info(&format!("5_gallery フォルダを作成しました: {}", gallery_dir.display()));
        }

        // 1_projects以下を再帰的に探索
        UI::info("1_projects 以下を探索しています...");
        let project_files = Self::scan_project_files(&projects_dir)?;
        
        if project_files.is_empty() {
            UI::warning("プロジェクトファイルが見つかりませんでした。");
            return Ok(());
        }

        // YYYYMMDD_xxx パターンのプロジェクトフォルダと成果物のマッピング
        let matches = Self::find_matching_outputs(&project_files)?;
        
        if matches.is_empty() {
            UI::warning("命名規則に従ったプロジェクト成果物が見つかりませんでした。");
            return Ok(());
        }

        UI::info(&format!("\n{} 件のプロジェクト成果物を発見しました。", matches.len()));
        
        // ショートカット作成
        let mut created = 0;
        let mut skipped = 0;
        
        for (project_name, target_file) in matches {
            let shortcut_path = gallery_dir.join(format!("{}.lnk", project_name));
            
            // 既に存在する場合はスキップ
            if shortcut_path.exists() {
                skipped += 1;
                continue;
            }

            // ショートカット作成
            if Self::create_shortcut(&target_file, &shortcut_path)? {
                created += 1;
                UI::info(&format!("  作成: {} -> {}", project_name, target_file.display()));
            }
        }

        UI::success(&format!("\nショートカット作成完了: {} 件作成、{} 件スキップ", created, skipped));
        Ok(())
    }

    /// 1_projects以下のファイルをスキャン
    fn scan_project_files(projects_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(projects_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                files.push(entry.into_path());
            }
        }
        
        Ok(files)
    }

    /// YYYYMMDD_projectname パターンのフォルダと成果物をマッチング
    fn find_matching_outputs(files: &[PathBuf]) -> Result<HashMap<String, PathBuf>> {
        let mut matches = HashMap::new();
        
        // メディアファイルの拡張子
        let media_extensions = [
            "mp4", "avi", "mov", "mkv", "wmv", "flv", "webm",
            "mp3", "wav", "flac", "aac", "ogg",
            "png", "jpg", "jpeg", "gif", "bmp", "webp",
            "pdf", "psd", "ai", "svg",
        ];
        
        for file in files {
            let file_name = match file.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };
            
            // 拡張子チェック
            let extension = file
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            if !media_extensions.contains(&extension.as_str()) {
                continue;
            }
            
            // YYYYMMDD_projectname パターンをチェック
            if let Some((date_part, project_part)) = Self::extract_project_pattern(&file_name) {
                // 親ディレクトリ名もチェック
                if let Some(parent_dir) = file.parent() {
                    if let Some(dir_name) = parent_dir.file_name() {
                        let dir_str = dir_name.to_string_lossy();
                        
                        // ディレクトリ名が YYYYMMDD_projectname 形式で、
                        // ファイル名のプロジェクト部分と一致する場合
                        if let Some((dir_date, dir_project)) = Self::extract_project_pattern(&dir_str) {
                            if dir_project == project_part {
                                let key = format!("{}_{}", dir_date, dir_project);
                                matches.insert(key, file.clone());
                                continue;
                            }
                        }
                        
                        // ディレクトリ名がプロジェクト名と部分一致する場合
                        if dir_str.contains(&project_part) {
                            let key = format!("{}_{}", date_part, project_part);
                            matches.insert(key, file.clone());
                        }
                    }
                }
            }
        }
        
        Ok(matches)
    }

    /// YYYYMMDD_projectname パターンから日付とプロジェクト名を抽出
    fn extract_project_pattern(name: &str) -> Option<(String, String)> {
        // 拡張子を除去
        let name_without_ext = if let Some(dot_pos) = name.rfind('.') {
            &name[..dot_pos]
        } else {
            name
        };
        
        let parts: Vec<&str> = name_without_ext.split('_').collect();
        if parts.len() < 2 {
            return None;
        }
        
        let date_part = parts[0];
        // YYYYMMDD 形式かチェック (8桁の数字)
        if date_part.len() == 8 && date_part.chars().all(|c| c.is_ascii_digit()) {
            let project_part = parts[1..].join("_");
            return Some((date_part.to_string(), project_part));
        }
        
        None
    }

    /// ショートカットを作成
    #[cfg(target_os = "windows")]
    fn create_shortcut(target: &Path, link_path: &Path) -> Result<bool> {
        use std::os::windows::fs::symlink_file;
        
        // Windowsではシンボリックリンクまたは.lnkファイルを作成
        // 権限の問題でシンボリックリンクが作れない場合もあるため、
        // 簡易的にハードリンクまたはコピーで対応
        match symlink_file(target, link_path) {
            Ok(_) => Ok(true),
            Err(_) => {
                // シンボリックリンクが失敗した場合、情報ファイルを作成
                let info = format!("Target: {}", target.display());
                fs::write(link_path, info)?;
                Ok(true)
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn create_shortcut(target: &Path, link_path: &Path) -> Result<bool> {
        use std::os::unix::fs::symlink;
        
        symlink(target, link_path)
            .with_context(|| format!("シンボリックリンクの作成に失敗: {:?} -> {:?}", link_path, target))?;
        Ok(true)
    }
}
