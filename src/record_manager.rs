use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use chrono::{Local, Datelike};
use crate::scanner::FileInfo;
use crate::naming::NamingRule;

/// recordフォルダの管理
pub struct RecordManager;

#[derive(Debug, Clone)]
pub enum RecordType {
    ScreenCapture,
    ScreenRecord,
    VoiceRecord,
}

impl RecordType {
    pub fn folder_name(&self) -> &'static str {
        match self {
            RecordType::ScreenCapture => "screen capture",
            RecordType::ScreenRecord => "screen record",
            RecordType::VoiceRecord => "voice record",
        }
    }

    pub fn naming_prefix(&self) -> &'static str {
        match self {
            RecordType::ScreenCapture => "screen-capture",
            RecordType::ScreenRecord => "screen-record",
            RecordType::VoiceRecord => "voice-record",
        }
    }

    #[allow(dead_code)]
    pub fn from_path(path: &Path) -> Option<Self> {
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("screen capture") || path_str.contains("screen-capture") {
            Some(RecordType::ScreenCapture)
        } else if path_str.contains("screen record") || path_str.contains("screen-record") {
            Some(RecordType::ScreenRecord)
        } else if path_str.contains("voice record") || path_str.contains("voice-record") {
            Some(RecordType::VoiceRecord)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct RecordFileAction {
    pub source: PathBuf,
    pub target: PathBuf,
    pub action_type: ActionType,
    #[allow(dead_code)]
    pub record_type: RecordType,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Move,
    Rename,
    #[allow(dead_code)]
    CreateFolder,
}

impl RecordManager {
    /// recordフォルダ内のファイルを整理
    pub fn organize_records(
        root: &PathBuf,
        dry_run: bool,
    ) -> Result<RecordOrganizationResult> {
        let record_base = root.join("0_inbox/record");
        
        if !record_base.exists() {
            return Ok(RecordOrganizationResult {
                actions: Vec::new(),
                created_folders: Vec::new(),
            });
        }

        let mut actions = Vec::new();
        let mut created_folders = Vec::new();

        // 各recordタイプを処理
        for record_type in [
            RecordType::ScreenCapture,
            RecordType::ScreenRecord,
            RecordType::VoiceRecord,
        ] {
            let record_path = record_base.join(record_type.folder_name());
            
            if !record_path.exists() {
                if !dry_run {
                    fs::create_dir_all(&record_path)
                        .with_context(|| format!("フォルダの作成に失敗: {:?}", record_path))?;
                }
                created_folders.push(record_path.clone());
                continue;
            }

            // ファイルをスキャン
            let files = Self::scan_record_folder(&record_path)?;

            for file in files {
                // 命名規則をチェック
                let needs_rename = !NamingRule::check_record_naming(&file.name);
                
                // 適切なフォルダを決定
                let target_folder = Self::determine_target_folder(
                    &file,
                    &record_path,
                    &record_type,
                )?;

                // フォルダが存在しない場合は作成
                if !target_folder.exists() {
                    if !dry_run {
                        fs::create_dir_all(&target_folder)
                            .with_context(|| format!("フォルダの作成に失敗: {:?}", target_folder))?;
                    }
                    created_folders.push(target_folder.clone());
                }

                // ファイル名を決定
                let target_filename = if needs_rename {
                    Self::generate_record_filename(&file, &record_type)?
                } else {
                    file.name.clone()
                };

                let target_path = target_folder.join(&target_filename);

                // 既に正しい場所にある場合はスキップ
                if file.path == target_path {
                    continue;
                }

                actions.push(RecordFileAction {
                    source: file.path.clone(),
                    target: target_path,
                    action_type: if needs_rename {
                        ActionType::Rename
                    } else {
                        ActionType::Move
                    },
                    record_type: record_type.clone(),
                });
            }
        }

        Ok(RecordOrganizationResult {
            actions,
            created_folders,
        })
    }

    /// recordフォルダをスキャン
    fn scan_record_folder(record_path: &Path) -> Result<Vec<FileInfo>> {
        use crate::scanner::DriveScanner;
        let all_files = DriveScanner::scan(record_path)?;
        Ok(all_files.into_iter().filter(|f| !f.is_dir).collect())
    }

    /// ファイルの日付に基づいて適切なフォルダを決定
    fn determine_target_folder(
        file: &FileInfo,
        record_path: &Path,
        _record_type: &RecordType,
    ) -> Result<PathBuf> {
        let now = Local::now();
        let file_date = file.modified;
        
        // 1年以上前かどうかチェック
        let one_year_ago = now.date_naive() - chrono::Duration::days(365);
        let file_date_naive = file_date.date_naive();
        
        if file_date_naive < one_year_ago {
            // YYYY/MM/ フォルダ
            let year = file_date.year();
            let month = format!("{:02}", file_date.month());
            Ok(record_path.join(format!("{}/{}", year, month)))
        } else {
            // MM/ フォルダ
            let month = format!("{:02}", file_date.month());
            Ok(record_path.join(&month))
        }
    }

    /// recordファイル名を生成
    fn generate_record_filename(
        file: &FileInfo,
        record_type: &RecordType,
    ) -> Result<String> {
        let extension = file.extension.clone();
        let timestamp = file.modified.format("%Y%m%d%H%M%S").to_string();
        Ok(format!("{}_{}.{}", timestamp, record_type.naming_prefix(), extension))
    }

    /// アクションを実行
    pub fn execute_actions(
        actions: &[RecordFileAction],
        dry_run: bool,
    ) -> Result<()> {
        for action in actions {
            if dry_run {
                match action.action_type {
                    ActionType::Move => {
                        println!("[DRY RUN] 移動: {:?} → {:?}", action.source, action.target);
                    }
                    ActionType::Rename => {
                        println!("[DRY RUN] リネーム&移動: {:?} → {:?}", action.source, action.target);
                    }
                    ActionType::CreateFolder => {
                        println!("[DRY RUN] フォルダ作成: {:?}", action.target.parent());
                    }
                }
            } else {
                // ターゲットディレクトリが存在しない場合は作成
                if let Some(parent) = action.target.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("ディレクトリの作成に失敗: {:?}", parent))?;
                    }
                }

                // ファイルを移動/リネーム
                fs::rename(&action.source, &action.target)
                    .with_context(|| format!("ファイルの移動に失敗: {:?} → {:?}", action.source, action.target))?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RecordOrganizationResult {
    pub actions: Vec<RecordFileAction>,
    pub created_folders: Vec<PathBuf>,
}

impl RecordOrganizationResult {
    pub fn print_report(&self) {
        println!("=== Record整理結果 ===");
        println!("実行アクション: {}個", self.actions.len());
        println!("作成フォルダ: {}個", self.created_folders.len());
        
        if !self.actions.is_empty() {
            println!("\n実行されるアクション:");
            for action in &self.actions {
                match action.action_type {
                    ActionType::Move => {
                        println!("  → 移動: {:?} → {:?}", action.source, action.target);
                    }
                    ActionType::Rename => {
                        println!("  → リネーム&移動: {:?} → {:?}", action.source, action.target);
                    }
                    ActionType::CreateFolder => {
                        println!("  → フォルダ作成: {:?}", action.target.parent());
                    }
                }
            }
        }
    }
}

