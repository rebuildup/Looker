use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Datelike, Local};

use crate::naming::NamingRule;
use crate::scanner::{DriveScanner, FileInfo};

/// recordフォルダの整理ロジック
pub struct RecordManager;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct RecordOptions {
    pub target_types: Vec<RecordType>,
    pub check_misplaced: bool,
}

impl Default for RecordOptions {
    fn default() -> Self {
        Self {
            target_types: Vec::new(),
            check_misplaced: true,
        }
    }
}

impl RecordOptions {
    pub fn includes(&self, record_type: &RecordType) -> bool {
        self.target_types.is_empty() || self.target_types.iter().any(|target| target == record_type)
    }
}

#[derive(Debug, Clone)]
pub struct RecordFileAction {
    pub source: PathBuf,
    pub target: PathBuf,
    pub action_type: ActionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Move,
    Rename,
    MoveToCorrectLocation,
}

#[derive(Debug)]
pub struct RecordOrganizationPlan {
    pub record_root: PathBuf,
    pub actions: Vec<RecordFileAction>,
    pub required_folders: BTreeSet<PathBuf>,
}

impl RecordOrganizationPlan {
    pub fn new(record_root: PathBuf) -> Self {
        Self {
            record_root,
            actions: Vec::new(),
            required_folders: BTreeSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty() && self.required_folders.is_empty()
    }

    pub fn register_folder<P: AsRef<Path>>(&mut self, folder: P) {
        let folder = folder.as_ref();
        if !folder.exists() {
            self.required_folders.insert(folder.to_path_buf());
        }
    }
}

impl RecordManager {
    /// recordフォルダを走査して必要なアクションを計画する
    pub fn plan(record_root: &Path, options: &RecordOptions) -> Result<RecordOrganizationPlan> {
        let mut plan = RecordOrganizationPlan::new(record_root.to_path_buf());

        if !record_root.exists() {
            plan.register_folder(record_root);
            return Ok(plan);
        }

        let root_files = Self::scan_record_folder(record_root)?;
        for file in root_files {
            let record_type = Self::guess_record_type(&file.path);
            if !options.includes(&record_type) {
                continue;
            }

            let record_path = record_root.join(record_type.folder_name());
            plan.register_folder(&record_path);

            let target_folder = Self::determine_target_folder(&file, &record_path, &record_type)?;
            plan.register_folder(&target_folder);

            let needs_rename = !NamingRule::check_record_naming(&file.name);
            let target_filename = if needs_rename {
                Self::generate_record_filename(&file, &record_type)?
            } else {
                file.name.clone()
            };

            let target_path = target_folder.join(target_filename);
            if file.path == target_path {
                continue;
            }

            plan.actions.push(RecordFileAction {
                source: file.path.clone(),
                target: target_path,
                action_type: if needs_rename {
                    ActionType::Rename
                } else {
                    ActionType::Move
                },
            });
        }

        for record_type in [
            RecordType::ScreenCapture,
            RecordType::ScreenRecord,
            RecordType::VoiceRecord,
        ] {
            if !options.includes(&record_type) {
                continue;
            }

            let record_path = record_root.join(record_type.folder_name());
            plan.register_folder(&record_path);

            if !record_path.exists() {
                continue;
            }

            let files = Self::scan_record_folder(&record_path)?;
            for file in files {
                let needs_rename = !NamingRule::check_record_naming(&file.name);
                let target_folder =
                    Self::determine_target_folder(&file, &record_path, &record_type)?;
                plan.register_folder(&target_folder);

                let target_filename = if needs_rename {
                    Self::generate_record_filename(&file, &record_type)?
                } else {
                    file.name.clone()
                };

                let target_path = target_folder.join(target_filename);

                if file.path == target_path {
                    continue;
                }

                plan.actions.push(RecordFileAction {
                    source: file.path.clone(),
                    target: target_path,
                    action_type: if needs_rename {
                        ActionType::Rename
                    } else {
                        ActionType::Move
                    },
                });
            }
        }

        if options.check_misplaced {
            let misplaced = Self::check_misplaced_files(record_root, options)?;
            for action in &misplaced {
                if let Some(parent) = action.target.parent() {
                    plan.register_folder(parent);
                }
            }
            plan.actions.extend(misplaced);
        }

        plan.actions
            .sort_by(|a, b| a.source.cmp(&b.source).then(a.target.cmp(&b.target)));

        Ok(plan)
    }

    /// 計画済みアクションを適用
    pub fn apply(plan: &RecordOrganizationPlan) -> Result<()> {
        for folder in &plan.required_folders {
            fs::create_dir_all(folder)
                .with_context(|| format!("フォルダ作成に失敗: {:?}", folder))?;
        }

        for action in &plan.actions {
            if let Some(parent) = action.target.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)
                    .with_context(|| format!("フォルダ作成に失敗: {:?}", parent))?;
            }

            fs::rename(&action.source, &action.target).with_context(|| {
                format!(
                    "ファイル操作に失敗: {:?} -> {:?}",
                    action.source, action.target
                )
            })?;
        }

        Ok(())
    }

    /// 取り違えファイルのチェック
    fn check_misplaced_files(
        record_base: &Path,
        options: &RecordOptions,
    ) -> Result<Vec<RecordFileAction>> {
        let mut actions = Vec::new();

        for record_type in [
            RecordType::ScreenCapture,
            RecordType::ScreenRecord,
            RecordType::VoiceRecord,
        ] {
            if !options.includes(&record_type) {
                continue;
            }

            let record_path = record_base.join(record_type.folder_name());

            if !record_path.exists() {
                continue;
            }

            let all_files = Self::scan_all_files_recursive(&record_path)?;

            for file in all_files {
                let correct_type = Self::guess_record_type(&file.path);

                if !options.includes(&correct_type) {
                    continue;
                }

                let current_prefix = Self::extract_naming_prefix(&file.name);
                let correct_prefix = correct_type.naming_prefix();
                let needs_move = record_type != correct_type;
                let needs_fix_name = !current_prefix.is_empty() && current_prefix != correct_prefix;
                let needs_rename = needs_fix_name || !NamingRule::check_record_naming(&file.name);

                if !needs_move && !needs_rename {
                    continue;
                }

                let target_record_path = if needs_move {
                    record_base.join(correct_type.folder_name())
                } else {
                    record_path.clone()
                };

                let target_folder =
                    Self::determine_target_folder(&file, &target_record_path, &correct_type)?;

                let target_filename = if needs_rename {
                    Self::generate_record_filename(&file, &correct_type)?
                } else {
                    file.name.clone()
                };

                let target_path = target_folder.join(target_filename);

                if file.path == target_path {
                    continue;
                }

                actions.push(RecordFileAction {
                    source: file.path.clone(),
                    target: target_path,
                    action_type: if needs_rename {
                        ActionType::Rename
                    } else if needs_move {
                        ActionType::MoveToCorrectLocation
                    } else {
                        ActionType::Move
                    },
                });
            }
        }

        Ok(actions)
    }

    /// 再帰的にファイルを取得
    fn scan_all_files_recursive(record_path: &Path) -> Result<Vec<FileInfo>> {
        let all_files = DriveScanner::scan(record_path)?;
        Ok(all_files.into_iter().filter(|f| !f.is_dir).collect())
    }

    /// record配下の直下ファイルのみを取得
    fn scan_record_folder(record_path: &Path) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        let entries = fs::read_dir(record_path)
            .with_context(|| format!("ディレクトリの読込に失敗: {:?}", record_path))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.is_dir() {
                continue;
            }

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let modified = metadata
                .modified()
                .map(chrono::DateTime::<chrono::Local>::from)
                .unwrap_or_else(|_| chrono::Local::now());

            files.push(FileInfo {
                path,
                name,
                extension,
                size: metadata.len(),
                modified,
                is_dir: false,
            });
        }
        Ok(files)
    }

    /// ファイル拡張子などからrecord種別を推定
    fn guess_record_type(file_path: &Path) -> RecordType {
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" => {
                return RecordType::ScreenCapture;
            }
            "mp4" | "avi" | "mov" | "mkv" | "wmv" | "flv" | "webm" | "m4v" => {
                return RecordType::ScreenRecord;
            }
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => {
                return RecordType::VoiceRecord;
            }
            _ => {}
        }

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name.contains("screen-capture") || file_name.contains("screenshot") {
            RecordType::ScreenCapture
        } else if file_name.contains("screen-record") || file_name.contains("recording") {
            RecordType::ScreenRecord
        } else if file_name.contains("voice-record") || file_name.contains("voice") {
            RecordType::VoiceRecord
        } else {
            RecordType::ScreenCapture
        }
    }

    /// ファイルの更新日時から配置先フォルダを決定
    fn determine_target_folder(
        file: &FileInfo,
        record_path: &Path,
        _record_type: &RecordType,
    ) -> Result<PathBuf> {
        let now = Local::now();
        let file_date = file.modified;

        let current_year = now.year();
        let file_year = file_date.year();
        let file_month = format!("{:02}", file_date.month());
        let year_month = format!("{}{}", file_year, file_month);

        if file_year < current_year {
            Ok(record_path.join(format!("{}/{}", file_year, year_month)))
        } else {
            Ok(record_path.join(&year_month))
        }
    }

    /// 既存ファイル名から prefix を抽出
    fn extract_naming_prefix(filename: &str) -> String {
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() >= 2 {
            let prefix_with_ext = parts[1];
            if let Some(dot_pos) = prefix_with_ext.rfind('.') {
                return prefix_with_ext[..dot_pos].to_string();
            }
            return prefix_with_ext.to_string();
        }
        String::new()
    }

    /// recordファイル名を生成
    fn generate_record_filename(file: &FileInfo, record_type: &RecordType) -> Result<String> {
        let extension = file.extension.clone();
        let timestamp = file.modified.format("%Y%m%d%H%M%S").to_string();
        if extension.is_empty() {
            Ok(format!("{}_{}", timestamp, record_type.naming_prefix()))
        } else {
            Ok(format!(
                "{}_{}.{}",
                timestamp,
                record_type.naming_prefix(),
                extension
            ))
        }
    }
}
