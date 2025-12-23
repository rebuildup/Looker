use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::ui::UI;

/// フォルダ構造を管理するマネージャー
pub struct StructureManager;

impl StructureManager {
    /// 標準フォルダ構造の定義
    fn standard_structure() -> Vec<&'static str> {
        vec![
            "0_inbox",
            "0_inbox/downloads",
            "0_inbox/record",
            "0_inbox/record/screen capture",
            "0_inbox/record/screen record",
            "0_inbox/record/voice record",
            "1_projects",
            "2_assets",
            "2_assets/footage",
            "2_assets/graphic",
            "2_assets/photo",
            "2_assets/illust",
            "2_assets/bgm",
            "2_assets/sfx",
            "3_docs",
            "3_docs/profile",
            "3_docs/collection",
            "3_docs/class",
            "3_docs/club",
            "3_docs/guide",
            "3_docs/family",
            "3_docs/icon",
            "3_docs/meme",
            "4_apps",
            "5_gallery",
            "9_archive",
        ]
    }

    /// 標準フォルダ構造を検証して不足しているフォルダを作成
    pub fn ensure_standard_structure(root: &Path) -> Result<()> {
        UI::info(&format!("ルートディレクトリ: {}", root.display()));
        UI::info("標準フォルダ構造を確認しています...\n");

        let structure = Self::standard_structure();
        let mut missing_folders = Vec::new();
        let mut existing_folders = Vec::new();

        // 既存のフォルダと不足しているフォルダを確認
        for folder_path in structure {
            let full_path = root.join(folder_path);
            if full_path.exists() {
                existing_folders.push(folder_path);
            } else {
                missing_folders.push(folder_path);
            }
        }

        // 結果を表示
        UI::success(&format!("既存のフォルダ: {} 件", existing_folders.len()));
        if !existing_folders.is_empty() && existing_folders.len() <= 10 {
            for folder in &existing_folders {
                UI::info(&format!("  ✓ {}", folder));
            }
        }

        if missing_folders.is_empty() {
            UI::success("\nすべてのフォルダが既に存在しています。");
            return Ok(());
        }

        UI::warning(&format!("\n不足しているフォルダ: {} 件", missing_folders.len()));
        for folder in &missing_folders {
            UI::warning(&format!("  ✗ {}", folder));
        }

        // 不足しているフォルダを作成
        UI::info(&format!("\n{} 件のフォルダを作成します...", missing_folders.len()));
        let mut created = 0;
        
        for folder_path in missing_folders {
            let full_path = root.join(folder_path);
            fs::create_dir_all(&full_path)
                .with_context(|| format!("フォルダの作成に失敗: {}", full_path.display()))?;
            UI::info(&format!("  作成: {}", folder_path));
            created += 1;
        }

        UI::success(&format!("\n{} 件のフォルダを作成しました。", created));
        Ok(())
    }

    /// フォルダ構造の検証のみ（作成はしない）
    #[allow(dead_code)]
    pub fn validate_structure(root: &Path) -> Result<StructureValidationResult> {
        let structure = Self::standard_structure();
        let mut missing = Vec::new();
        let mut existing = Vec::new();

        for folder_path in structure {
            let full_path = root.join(folder_path);
            if full_path.exists() {
                existing.push(folder_path.to_string());
            } else {
                missing.push(folder_path.to_string());
            }
        }

        Ok(StructureValidationResult {
            total: existing.len() + missing.len(),
            existing,
            missing,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct StructureValidationResult {
    pub total: usize,
    pub existing: Vec<String>,
    pub missing: Vec<String>,
}

impl StructureValidationResult {
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }

    #[allow(dead_code)]
    pub fn completion_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.existing.len() as f64 / self.total as f64) * 100.0
    }
}
