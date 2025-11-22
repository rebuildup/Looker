use std::path::PathBuf;
use anyhow::Result;

/// 標準フォルダ構造の定義
pub struct FolderStructure;

impl FolderStructure {
    /// ルートディレクトリの標準構造を返す
    pub fn get_standard_structure() -> Vec<StandardPath> {
        vec![
            StandardPath::new("0_inbox", PathType::Inbox),
            StandardPath::new("0_inbox/downloads", PathType::InboxDownloads),
            StandardPath::new("0_inbox/record", PathType::InboxRecord),
            StandardPath::new("0_inbox/record/screen capture", PathType::RecordScreenCapture),
            StandardPath::new("0_inbox/record/screen record", PathType::RecordScreenRecord),
            StandardPath::new("0_inbox/record/voice record", PathType::RecordVoice),
            StandardPath::new("1_projects", PathType::Projects),
            StandardPath::new("2_assets", PathType::Assets),
            StandardPath::new("2_assets/footage", PathType::AssetsFootage),
            StandardPath::new("2_assets/graphic", PathType::AssetsGraphic),
            StandardPath::new("2_assets/photo", PathType::AssetsPhoto),
            StandardPath::new("2_assets/illust", PathType::AssetsIllust),
            StandardPath::new("2_assets/bgm", PathType::AssetsBgm),
            StandardPath::new("2_assets/sfx", PathType::AssetsSfx),
            StandardPath::new("3_docs", PathType::Docs),
            StandardPath::new("3_docs/plofile", PathType::DocsPlofile),
            StandardPath::new("3_docs/collection", PathType::DocsCollection),
            StandardPath::new("3_docs/class", PathType::DocsClass),
            StandardPath::new("3_docs/cclub", PathType::DocsCclub),
            StandardPath::new("3_docs/guide", PathType::DocsGuide),
            StandardPath::new("3_docs/family", PathType::DocsFamily),
            StandardPath::new("3_docs/icon", PathType::DocsIcon),
            StandardPath::new("3_docs/meme", PathType::DocsMeme),
            StandardPath::new("4_apps", PathType::Apps),
            StandardPath::new("5_gollery", PathType::Gallery),
            StandardPath::new("9_archive", PathType::Archive),
        ]
    }

    /// 指定されたパスが標準構造に適合しているかチェック
    pub fn validate_structure(root: &PathBuf) -> Result<ValidationResult> {
        let mut missing = Vec::new();
        let mut existing = Vec::new();

        for standard_path in Self::get_standard_structure() {
            let full_path = root.join(&standard_path.path);
            if full_path.exists() {
                existing.push(standard_path.path.clone());
            } else {
                missing.push(standard_path.path.clone());
            }
        }

        Ok(ValidationResult {
            missing,
            existing,
            root: root.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct StandardPath {
    pub path: String,
    #[allow(dead_code)]
    pub path_type: PathType,
}

impl StandardPath {
    pub fn new(path: &str, path_type: PathType) -> Self {
        Self {
            path: path.to_string(),
            path_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PathType {
    Inbox,
    InboxDownloads,
    InboxRecord,
    RecordScreenCapture,
    RecordScreenRecord,
    RecordVoice,
    Projects,
    Assets,
    AssetsFootage,
    AssetsGraphic,
    AssetsPhoto,
    AssetsIllust,
    AssetsBgm,
    AssetsSfx,
    Docs,
    DocsPlofile,
    DocsCollection,
    DocsClass,
    DocsCclub,
    DocsGuide,
    DocsFamily,
    DocsIcon,
    DocsMeme,
    Apps,
    Gallery,
    Archive,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub missing: Vec<String>,
    pub existing: Vec<String>,
    pub root: PathBuf,
}

impl ValidationResult {
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.missing.is_empty()
    }

    pub fn print_report(&self) {
        println!("=== フォルダ構造検証結果 ===");
        println!("ルート: {:?}", self.root);
        println!("\n存在するフォルダ: {}個", self.existing.len());
        for path in &self.existing {
            println!("  ✓ {}", path);
        }
        println!("\n不足しているフォルダ: {}個", self.missing.len());
        for path in &self.missing {
            println!("  ✗ {}", path);
        }
    }
}

