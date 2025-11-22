use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::recommender::FileRecommendation;
use crate::structure::FolderStructure;

/// 自動実行機能（ドライラン/実行モード）
pub struct Executor {
    dry_run: bool,
}

impl Executor {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// 推奨配置に基づいてファイルを移動
    pub fn execute_recommendations(
        &self,
        recommendations: &[FileRecommendation],
    ) -> Result<ExecutionResult> {
        let mut moved = Vec::new();
        let mut failed = Vec::new();

        for rec in recommendations {
            let source = &rec.file.path;
            let target = &rec.recommendation.target_path;

            // 既に正しい場所にある場合はスキップ
            if source.parent() == target.parent() {
                continue;
            }

            // ターゲットディレクトリが存在しない場合は作成
            if let Some(parent) = target.parent() {
                if !parent.exists() {
                    if self.dry_run {
                        println!("[DRY RUN] ディレクトリを作成: {:?}", parent);
                    } else {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("ディレクトリの作成に失敗: {:?}", parent))?;
                    }
                }
            }

            // ファイルを移動
            if self.dry_run {
                println!("[DRY RUN] 移動: {:?} → {:?}", source, target);
                moved.push((source.clone(), target.clone()));
            } else {
                match fs::rename(source, target) {
                    Ok(_) => {
                        moved.push((source.clone(), target.clone()));
                    }
                    Err(e) => {
                        failed.push((source.clone(), target.clone(), e.to_string()));
                    }
                }
            }
        }

        Ok(ExecutionResult { moved, failed })
    }

    /// 標準フォルダ構造を作成
    pub fn create_standard_structure(&self, root: &PathBuf) -> Result<()> {
        let structure = FolderStructure::get_standard_structure();

        for standard_path in structure {
            let full_path = root.join(&standard_path.path);

            if full_path.exists() {
                continue;
            }

            if self.dry_run {
                println!("[DRY RUN] フォルダを作成: {:?}", full_path);
            } else {
                fs::create_dir_all(&full_path)
                    .with_context(|| format!("フォルダの作成に失敗: {:?}", full_path))?;
            }
        }

        Ok(())
    }

    /// 命名規則に従ってファイルをリネーム
    #[allow(dead_code)]
    pub fn rename_files(
        &self,
        files: &[(PathBuf, String)], // (現在のパス, 新しい名前)
    ) -> Result<()> {
        for (path, new_name) in files {
            let parent = path.parent()
                .ok_or_else(|| anyhow::anyhow!("親ディレクトリが取得できません: {:?}", path))?;
            let new_path = parent.join(new_name);

            if self.dry_run {
                println!("[DRY RUN] リネーム: {:?} → {:?}", path, new_path);
            } else {
                fs::rename(path, &new_path)
                    .with_context(|| format!("リネームに失敗: {:?} → {:?}", path, new_path))?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub moved: Vec<(PathBuf, PathBuf)>,
    pub failed: Vec<(PathBuf, PathBuf, String)>,
}

impl ExecutionResult {
    pub fn print_report(&self) {
        println!("=== 実行結果 ===");
        println!("移動成功: {}個", self.moved.len());
        for (source, target) in &self.moved {
            println!("  ✓ {:?} → {:?}", source, target);
        }

        if !self.failed.is_empty() {
            println!("\n移動失敗: {}個", self.failed.len());
            for (source, target, error) in &self.failed {
                println!("  ✗ {:?} → {:?}", source, target);
                println!("    エラー: {}", error);
            }
        }
    }
}

