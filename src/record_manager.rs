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
    MoveToCorrectLocation, // 間違った場所から正しい場所へ移動
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

        // まず、recordフォルダ直下のファイルを処理（recordタイプが不明な場合）
        let root_files = Self::scan_record_folder(&record_base)?;
        if !root_files.is_empty() {
            println!("recordフォルダ直下に {}個のファイルを発見しました", root_files.len());
        }
        
        // recordフォルダ直下のファイルを処理（recordタイプを推測）
        for file in root_files {
            // ファイル名やパスからrecordタイプを推測
            let record_type = Self::guess_record_type(&file.path);
            
            let record_path = record_base.join(record_type.folder_name());
            
            // フォルダが存在しない場合は作成
            if !record_path.exists() {
                if !dry_run {
                    fs::create_dir_all(&record_path)
                        .with_context(|| format!("フォルダの作成に失敗: {:?}", record_path))?;
                }
                created_folders.push(record_path.clone());
            }
            
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
            
            // 命名規則をチェック
            let needs_rename = !NamingRule::check_record_naming(&file.name);
            
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

        // 各recordタイプのサブフォルダ内のファイルを処理
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

            // ファイルをスキャン（直下のファイルのみ）
            let files = Self::scan_record_folder(&record_path)?;
            
            if !files.is_empty() {
                println!("{} フォルダに {}個のファイルを発見しました", record_type.folder_name(), files.len());
            }

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

        // 既に配置されているファイルが正しい場所にあるかチェック
        let misplaced_files = Self::check_misplaced_files(&record_base)?;
        actions.extend(misplaced_files);

        Ok(RecordOrganizationResult {
            actions,
            created_folders,
        })
    }

    /// 間違った場所に配置されているファイルをチェック
    fn check_misplaced_files(record_base: &Path) -> Result<Vec<RecordFileAction>> {
        let mut actions = Vec::new();
        
        println!("間違った配置のファイルをチェック中...");
        
        // 各recordタイプのサブフォルダをチェック
        for record_type in [
            RecordType::ScreenCapture,
            RecordType::ScreenRecord,
            RecordType::VoiceRecord,
        ] {
            let record_path = record_base.join(record_type.folder_name());
            
            if !record_path.exists() {
                println!("  {} フォルダは存在しません: {:?}", record_type.folder_name(), record_path);
                continue;
            }
            
            // 再帰的にスキャン（サブフォルダ内も含む）
            let all_files = Self::scan_all_files_recursive(&record_path)?;
            println!("  {} フォルダ: {}個のファイルをスキャン", record_type.folder_name(), all_files.len());
            
            for file in all_files {
                // ファイルの拡張子から正しいrecordタイプを判定
                let correct_type = Self::guess_record_type(&file.path);
                
                // ファイル名から現在のプレフィックスを抽出
                let current_naming_prefix = Self::extract_naming_prefix(&file.name);
                let correct_naming_prefix = correct_type.naming_prefix();
                
                println!("    ファイル: {} → 正しいタイプ: {:?}", file.name, correct_type.folder_name());
                println!("      現在のプレフィックス: '{}', 正しいプレフィックス: '{}'", current_naming_prefix, correct_naming_prefix);
                
                // 現在のrecordタイプと一致しない場合は移動が必要
                // RecordTypeはClone可能なので、比較するためにmatchで判定
                let needs_move = match (&record_type, &correct_type) {
                    (RecordType::ScreenCapture, RecordType::ScreenCapture) => false,
                    (RecordType::ScreenRecord, RecordType::ScreenRecord) => false,
                    (RecordType::VoiceRecord, RecordType::VoiceRecord) => false,
                    _ => true,
                };
                
                // ファイル名のプレフィックスが間違っている場合も修正が必要
                let needs_fix_name = !current_naming_prefix.is_empty() 
                    && current_naming_prefix != correct_naming_prefix;
                
                println!("      移動必要: {}, 名前修正必要: {}", needs_move, needs_fix_name);
                
                if needs_move || needs_fix_name {
                    if needs_move {
                        println!("      → 移動が必要: {} → {}", record_type.folder_name(), correct_type.folder_name());
                    }
                    if needs_fix_name {
                        println!("      → ファイル名修正が必要: '{}' → '{}'", current_naming_prefix, correct_naming_prefix);
                    }
                    
                    // 移動が必要な場合は正しいrecordタイプのフォルダに移動
                    let target_record_path = if needs_move {
                        record_base.join(correct_type.folder_name())
                    } else {
                        record_path.clone()
                    };
                    
                    // 適切なフォルダを決定
                    let target_folder = Self::determine_target_folder(
                        &file,
                        &target_record_path,
                        &correct_type,
                    )?;
                    
                    // ファイル名を決定
                    // recordタイプが間違っている場合、またはファイル名のプレフィックスが間違っている場合は、ファイル名も必ず修正する
                    let needs_rename = !NamingRule::check_record_naming(&file.name) 
                        || needs_fix_name;
                    
                    println!("        リネーム必要: {}", needs_rename);
                    
                    let target_filename = if needs_rename {
                        let new_name = Self::generate_record_filename(&file, &correct_type)?;
                        println!("        新しいファイル名: {}", new_name);
                        new_name
                    } else {
                        file.name.clone()
                    };
                    
                    let target_path = target_folder.join(&target_filename);
                    
                    // 既に正しい場所にある場合はスキップ
                    if file.path == target_path {
                        println!("        既に正しい場所にあります: {:?}", target_path);
                        continue;
                    }
                    
                    println!("        アクション追加: {:?} → {:?}", file.path, target_path);
                    
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
                        record_type: correct_type.clone(),
                    });
                } else {
                    println!("      → 正しい場所にあります");
                }
            }
        }
        
        println!("間違った配置のチェック完了: {}個のアクション", actions.len());
        Ok(actions)
    }

    /// 再帰的にすべてのファイルをスキャン（サブフォルダ内も含む）
    fn scan_all_files_recursive(record_path: &Path) -> Result<Vec<FileInfo>> {
        use crate::scanner::DriveScanner;
        let all_files = DriveScanner::scan(record_path)?;
        Ok(all_files.into_iter().filter(|f| !f.is_dir).collect())
    }

    /// recordフォルダをスキャン（直下のファイルのみ）
    fn scan_record_folder(record_path: &Path) -> Result<Vec<FileInfo>> {
        use std::fs;
        let mut files = Vec::new();
        
        // 直下のファイルのみをスキャン（サブフォルダは無視）
        let entries = fs::read_dir(record_path)
            .with_context(|| format!("ディレクトリの読み込みに失敗: {:?}", record_path))?;
        
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("エントリ読み込みエラー: {}", e);
                    continue;
                }
            };
            
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("メタデータ取得エラー: {} ({:?})", e, entry.path());
                    continue;
                }
            };
            
            // ディレクトリはスキップ（既に整理済みのフォルダを無視）
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
                .map(|t| chrono::DateTime::<chrono::Local>::from(t))
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

    /// ファイルパスからrecordタイプを推測（拡張子とファイル名から判定）
    fn guess_record_type(file_path: &Path) -> RecordType {
        // まず拡張子で判定
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // 拡張子による判定
        match extension.as_str() {
            // 画像ファイル → screen-capture
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" => {
                return RecordType::ScreenCapture;
            }
            // 動画ファイル → screen-record
            "mp4" | "avi" | "mov" | "mkv" | "wmv" | "flv" | "webm" | "m4v" => {
                return RecordType::ScreenRecord;
            }
            // 音声ファイル → voice-record
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => {
                return RecordType::VoiceRecord;
            }
            _ => {}
        }
        
        // 拡張子で判定できない場合はファイル名から推測
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        if file_name.contains("screen-capture") || file_name.contains("screenshot") || file_name.contains("capture") {
            RecordType::ScreenCapture
        } else if file_name.contains("screen-record") || file_name.contains("recording") || file_name.contains("record") {
            RecordType::ScreenRecord
        } else if file_name.contains("voice-record") || file_name.contains("voice") || file_name.contains("audio") {
            RecordType::VoiceRecord
        } else {
            // デフォルトはscreen capture（画像ファイルが多いため）
            RecordType::ScreenCapture
        }
    }

    /// ファイルの日付に基づいて適切なフォルダを決定
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
            // 去年以前: YYYY/YYYYMM/ フォルダ
            Ok(record_path.join(format!("{}/{}", file_year, year_month)))
        } else {
            // 今年: YYYYMM/ フォルダ
            Ok(record_path.join(&year_month))
        }
    }

    /// ファイル名から命名プレフィックスを抽出
    fn extract_naming_prefix(filename: &str) -> String {
        // YYYYMMDDHHMMSS_[prefix].[extension] の形式から [prefix] を抽出
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() >= 2 {
            // 拡張子を除いた部分を取得
            let prefix_with_ext = parts[1];
            if let Some(dot_pos) = prefix_with_ext.rfind('.') {
                return prefix_with_ext[..dot_pos].to_string();
            }
            return prefix_with_ext.to_string();
        }
        // 命名規則に従っていないファイルの場合は空文字列を返す
        String::new()
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
                    ActionType::MoveToCorrectLocation => {
                        println!("[DRY RUN] 正しい場所へ移動: {:?} → {:?}", action.source, action.target);
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
        use crate::ui::UI;
        UI::section("Record整理結果");
        UI::key_value("実行アクション", &format!("{}個", self.actions.len()));
        UI::key_value("作成フォルダ", &format!("{}個", self.created_folders.len()));
        
        if !self.actions.is_empty() {
            println!("\n実行されるアクション:");
            for (idx, action) in self.actions.iter().enumerate() {
                match action.action_type {
                    ActionType::Move => {
                        println!("  {}. 移動: {:?} → {:?}", idx + 1, action.source, action.target);
                    }
                    ActionType::Rename => {
                        println!("  {}. リネーム&移動: {:?} → {:?}", idx + 1, action.source, action.target);
                    }
                    ActionType::MoveToCorrectLocation => {
                        println!("  {}. 正しい場所へ移動: {:?} → {:?}", idx + 1, action.source, action.target);
                    }
                    ActionType::CreateFolder => {
                        println!("  {}. フォルダ作成: {:?}", idx + 1, action.target.parent().unwrap_or(&std::path::PathBuf::new()));
                    }
                }
            }
        }
        
        if !self.created_folders.is_empty() {
            println!("\n作成されるフォルダ:");
            for folder in &self.created_folders {
                println!("  → {:?}", folder);
            }
        }
    }
}

