use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Local};

use crate::naming::NamingRule;
use crate::scanner::{DriveScanner, FileInfo};

/// Record フォルダを整理するメインロジック
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
        self.target_types.is_empty()
            || self
                .target_types
                .iter()
                .any(|target| target == record_type)
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
    const RECORD_TYPES: [RecordType; 3] = [
        RecordType::ScreenCapture,
        RecordType::ScreenRecord,
        RecordType::VoiceRecord,
    ];

    /// Record フォルダを走査して必要なアクションを組み立てる
    pub fn plan(record_root: &Path, options: &RecordOptions) -> Result<RecordOrganizationPlan> {
        let mut plan = RecordOrganizationPlan::new(record_root.to_path_buf());

        if !record_root.exists() {
            plan.register_folder(record_root);
            return Ok(plan);
        }

        // これから作成するターゲットパスをすべて記録し、重複しないようにする
        let mut planned_targets: BTreeSet<PathBuf> = BTreeSet::new();

        // 1. record_root 直下のファイルを整理
        let root_files = Self::scan_record_folder(record_root)?;
        for file in root_files {
            let record_type = Self::guess_record_type(&file.path);
            if !options.includes(&record_type) {
                continue;
            }

            let record_path = record_root.join(record_type.folder_name());
            plan.register_folder(&record_path);

            let target_folder =
                Self::determine_target_folder(&file, &record_path, &record_type)?;
            plan.register_folder(&target_folder);

            let needs_rename = !NamingRule::check_record_naming(&file.name);
            let base_filename = if needs_rename {
                Self::generate_record_filename(&file, &record_type)?
            } else {
                file.name.clone()
            };

            let target_path =
                Self::unique_target_path(&target_folder, &base_filename, &mut planned_targets)?;
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

        // 2. 各 record 種別の直下ファイルを整理
        for record_type in Self::RECORD_TYPES {
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

                let base_filename = if needs_rename {
                    Self::generate_record_filename(&file, &record_type)?
                } else {
                    file.name.clone()
                };

                let target_path =
                    Self::unique_target_path(&target_folder, &base_filename, &mut planned_targets)?;
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

        // 3. 誤配置ファイルと規定外サブフォルダ配下を整理
        if options.check_misplaced {
            let misplaced =
                Self::check_misplaced_files(record_root, options, &mut planned_targets)?;
            for action in &misplaced {
                if let Some(parent) = action.target.parent() {
                    plan.register_folder(parent);
                }
            }
            plan.actions.extend(misplaced);
        }

        // 4. 見やすさのためソート
        plan.actions
            .sort_by(|a, b| a.source.cmp(&b.source).then(a.target.cmp(&b.target)));

        Ok(plan)
    }

    /// プラン済みアクションを適用
    pub fn apply(plan: &RecordOrganizationPlan) -> Result<()> {
        use crate::ui::UI;

        // 1. 最終防衛線: ターゲット重複と既存ファイルへの上書きを検査
        let mut seen_targets = BTreeSet::new();
        for action in &plan.actions {
            if !seen_targets.insert(action.target.clone()) {
                return Err(anyhow!(
                    "実行予定のターゲットパスが重複しています: {:?}",
                    action.target
                ));
            }

            if action.target.exists() {
                return Err(anyhow!(
                    "既存のファイルへ適用しようとしました: {:?} -> {:?}",
                    action.source,
                    action.target
                ));
            }
        }

        // 2. 必要なフォルダ作成
        let folder_count = plan.required_folders.len();
        if folder_count > 0 {
            UI::info(&format!("フォルダを作成中... ({} 件)", folder_count));
        }
        for (idx, folder) in plan.required_folders.iter().enumerate() {
            fs::create_dir_all(folder)
                .with_context(|| format!("フォルダ作成に失敗: {:?}", folder))?;
            UI::info(&format!("  [{}/{}] 作成: {}", idx + 1, folder_count, folder.display()));
        }

        // 3. アクションを順に適用
        let action_count = plan.actions.len();
        if action_count > 0 {
            UI::info(&format!("\nファイルを移動中... ({} 件)", action_count));
        }
        for (idx, action) in plan.actions.iter().enumerate() {
            if let Some(parent) = action.target.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)
                    .with_context(|| format!("フォルダ作成に失敗: {:?}", parent))?;
            }

            fs::rename(&action.source, &action.target).with_context(|| {
                format!(
                    "ファイル移動に失敗: {:?} -> {:?}",
                    action.source, action.target
                )
            })?;
            
            UI::info(&format!(
                "  [{}/{}] {} -> {}",
                idx + 1,
                action_count,
                action.source.file_name().unwrap_or_default().to_string_lossy(),
                action.target.display()
            ));
        }

        // 4. 規定外サブフォルダで空になったものを片付ける
        UI::info("\n空フォルダをクリーンアップ中...");
        Self::cleanup_non_standard_empty_dirs(&plan.record_root)?;

        UI::success("\nすべての処理が完了しました。");
        Ok(())
    }

    /// 誤配置ファイル・規定外フォルダ配下のファイルを検出
    fn check_misplaced_files(
        record_base: &Path,
        options: &RecordOptions,
        planned_targets: &mut BTreeSet<PathBuf>,
    ) -> Result<Vec<RecordFileAction>> {
        let mut actions = Vec::new();

        // 1. 各 record 種別配下を再帰的にチェック
        for record_type in Self::RECORD_TYPES {
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
                let needs_fix_name =
                    !current_prefix.is_empty() && current_prefix != correct_prefix;
                let needs_rename =
                    needs_fix_name || !NamingRule::check_record_naming(&file.name);

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

                let base_filename = if needs_rename {
                    Self::generate_record_filename(&file, &correct_type)?
                } else {
                    file.name.clone()
                };

                let target_path =
                    Self::unique_target_path(&target_folder, &base_filename, planned_targets)?;
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

        // 2. record_root 直下の「規定外サブフォルダ」配下を整理
        let entries = fs::read_dir(record_base)
            .with_context(|| format!("record ルートの走査に失敗: {:?}", record_base))?;

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if !metadata.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let is_standard = Self::RECORD_TYPES
                .iter()
                .any(|kind| kind.folder_name() == name);
            if is_standard {
                continue;
            }

            let sub_root = entry.path();
            let sub_files = Self::scan_all_files_recursive(&sub_root)?;

            for file in sub_files {
                let correct_type = Self::guess_record_type(&file.path);
                if !options.includes(&correct_type) {
                    continue;
                }

                let target_record_path = record_base.join(correct_type.folder_name());
                let target_folder =
                    Self::determine_target_folder(&file, &target_record_path, &correct_type)?;

                let current_prefix = Self::extract_naming_prefix(&file.name);
                let correct_prefix = correct_type.naming_prefix();
                let needs_fix_name =
                    !current_prefix.is_empty() && current_prefix != correct_prefix;
                let needs_rename =
                    needs_fix_name || !NamingRule::check_record_naming(&file.name);

                let base_filename = if needs_rename {
                    Self::generate_record_filename(&file, &correct_type)?
                } else {
                    file.name.clone()
                };

                let target_path =
                    Self::unique_target_path(&target_folder, &base_filename, planned_targets)?;
                if file.path == target_path {
                    continue;
                }

                actions.push(RecordFileAction {
                    source: file.path.clone(),
                    target: target_path,
                    action_type: if needs_rename {
                        ActionType::Rename
                    } else {
                        ActionType::MoveToCorrectLocation
                    },
                });
            }
        }

        Ok(actions)
    }

    /// 再帰的にファイルのみ取得
    fn scan_all_files_recursive(record_path: &Path) -> Result<Vec<FileInfo>> {
        let all_files = DriveScanner::scan(record_path)?;
        Ok(all_files.into_iter().filter(|info| !info.is_dir).collect())
    }

    /// 指定フォルダ直下のファイルのみ取得
    fn scan_record_folder(record_path: &Path) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        let entries = fs::read_dir(record_path)
            .with_context(|| format!("ディレクトリの読み取りに失敗: {:?}", record_path))?;

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if metadata.is_dir() {
                continue;
            }

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
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

    /// 拡張子やファイル名から record 種別を推定
    fn guess_record_type(file_path: &Path) -> RecordType {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
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
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name.contains("screen-capture") || file_name.contains("screenshot") {
            RecordType::ScreenCapture
        } else if file_name.contains("screen-record") || file_name.contains("recording") {
            RecordType::ScreenRecord
        } else if file_name.contains("voice-record") || file_name.contains("voice") {
            RecordType::VoiceRecord
        } else {
            // 不明な場合は screen capture とみなす（元の仕様を踏襲）
            RecordType::ScreenCapture
        }
    }

    /// 更新日時から年/月フォルダを決定
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

    /// ファイル名から prefix（screen-capture 等）を抽出
    /// 末尾の -2, -3 などのサフィックスは無視する
    fn extract_naming_prefix(filename: &str) -> String {
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() >= 2 {
            let prefix_with_ext = parts[1];
            let mut prefix = if let Some(dot_pos) = prefix_with_ext.rfind('.') {
                prefix_with_ext[..dot_pos].to_string()
            } else {
                prefix_with_ext.to_string()
            };

            if let Some((head, tail)) = prefix.rsplit_once('-') {
                if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
                    prefix = head.to_string();
                }
            }

            return prefix;
        }
        String::new()
    }

    /// record ファイル名を生成（サフィックスなしのベース名）
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

    /// 同じフォルダ内で一意になるターゲットパスを決定する
    ///
    /// - ベース名で空いていればそのまま使う
    /// - 既に存在していれば `-2`, `-3`, ... のように番号を振って空き名を探す
    fn unique_target_path(
        target_folder: &Path,
        base_filename: &str,
        planned_targets: &mut BTreeSet<PathBuf>,
    ) -> Result<PathBuf> {
        // まずはベース名のまま試す
        let mut candidate = target_folder.join(base_filename);
        if !planned_targets.contains(&candidate) && !candidate.exists() {
            planned_targets.insert(candidate.clone());
            return Ok(candidate);
        }

        // ベース名を {stem}.{ext} に分割
        let (stem, ext) = match base_filename.rsplit_once('.') {
            Some((s, e)) => (s.to_string(), Some(e.to_string())),
            None => (base_filename.to_string(), None),
        };

        // 2 から番号を振って空き名を探す
        let mut index: u32 = 2;
        loop {
            let new_name = match &ext {
                Some(ext) => format!("{stem}-{index}.{ext}"),
                None => format!("{stem}-{index}"),
            };
            candidate = target_folder.join(&new_name);

            if !planned_targets.contains(&candidate) && !candidate.exists() {
                planned_targets.insert(candidate.clone());
                return Ok(candidate);
            }

            index = index
                .checked_add(1)
                .ok_or_else(|| anyhow!("重複回避用の連番生成に失敗しました"))?;
        }
    }

    /// record_root 直下の規定外サブフォルダで、空になったものを削除
    fn cleanup_non_standard_empty_dirs(record_root: &Path) -> Result<()> {
        let entries = match fs::read_dir(record_root) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if !metadata.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let is_standard = Self::RECORD_TYPES
                .iter()
                .any(|kind| kind.folder_name() == name);
            if is_standard {
                continue;
            }

            let path = entry.path();
            // 中身が空（サブディレクトリも空）であれば削除する
            let _ = Self::remove_empty_dirs_recursive(&path)?;
        }

        Ok(())
    }

    /// 空のディレクトリツリーなら再帰的に削除する
    fn remove_empty_dirs_recursive(path: &Path) -> Result<bool> {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return Ok(false),
        };

        let mut is_empty = true;

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    is_empty = false;
                    continue;
                }
            };

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    is_empty = false;
                    continue;
                }
            };

            if metadata.is_dir() {
                let child_empty = Self::remove_empty_dirs_recursive(&entry.path())?;
                if !child_empty {
                    is_empty = false;
                }
            } else {
                // ファイルが残っているので空ではない
                is_empty = false;
            }
        }

        if is_empty {
            fs::remove_dir(path)
                .with_context(|| format!("空ディレクトリの削除に失敗: {:?}", path))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

