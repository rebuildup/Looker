use regex::Regex;
use std::path::Path;

/// 命名規則のチェックとリネーム
pub struct NamingRule;

impl NamingRule {
    /// プロジェクトファイルの命名規則パターン
    /// YYYYMMDD_[project]_[item]_[option].[extension]
    pub fn check_project_naming(filename: &str) -> bool {
        let pattern = r"^\d{8}_[^_]+_[^_]+(?:_[^_]+)?\.[^.]+$";
        let re = Regex::new(pattern).unwrap();
        re.is_match(filename)
    }

    /// record内のファイル命名規則パターン
    /// YYYYMMDDHHMMSS_[screen-capture/screen-record/voice-record].[extension]
    pub fn check_record_naming(filename: &str) -> bool {
        let pattern = r"^\d{14}_(screen-capture|screen-record|voice-record)\.[^.]+$";
        let re = Regex::new(pattern).unwrap();
        re.is_match(filename)
    }

    /// ファイル名が適切な命名規則に従っているかチェック
    pub fn validate_filename(path: &Path, is_record: bool) -> NamingValidation {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if is_record {
            if Self::check_record_naming(filename) {
                NamingValidation::Valid
            } else {
                NamingValidation::InvalidRecord {
                    current: filename.to_string(),
                    suggested: Self::suggest_record_name(path),
                }
            }
        } else {
            if Self::check_project_naming(filename) {
                NamingValidation::Valid
            } else {
                NamingValidation::InvalidProject {
                    current: filename.to_string(),
                    suggested: Self::suggest_project_name(path),
                }
            }
        }
    }

    /// recordファイルの推奨名を生成
    fn suggest_record_name(path: &Path) -> String {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let record_type = if path.to_string_lossy().contains("screen-capture") {
            "screen-capture"
        } else if path.to_string_lossy().contains("screen-record") {
            "screen-record"
        } else if path.to_string_lossy().contains("voice-record") {
            "voice-record"
        } else {
            "screen-capture" // デフォルト
        };

        let now = chrono::Local::now();
        format!("{}_{}.{}", now.format("%Y%m%d%H%M%S"), record_type, extension)
    }

    /// プロジェクトファイルの推奨名を生成
    fn suggest_project_name(path: &Path) -> String {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let now = chrono::Local::now();
        format!("{}_{}_item.{}", now.format("%Y%m%d"), stem, extension)
    }
}

#[derive(Debug, Clone)]
pub enum NamingValidation {
    Valid,
    InvalidProject {
        current: String,
        suggested: String,
    },
    InvalidRecord {
        current: String,
        suggested: String,
    },
}

impl NamingValidation {
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        matches!(self, NamingValidation::Valid)
    }

    pub fn print_report(&self) {
        match self {
            NamingValidation::Valid => {
                println!("✓ 命名規則に適合しています");
            }
            NamingValidation::InvalidProject { current, suggested } => {
                println!("✗ プロジェクトファイルの命名規則に適合していません");
                println!("  現在: {}", current);
                println!("  推奨: {}", suggested);
            }
            NamingValidation::InvalidRecord { current, suggested } => {
                println!("✗ recordファイルの命名規則に適合していません");
                println!("  現在: {}", current);
                println!("  推奨: {}", suggested);
            }
        }
    }
}

