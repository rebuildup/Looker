use inquire::{Select, Confirm};
use std::path::PathBuf;
use anyhow::Result;
use crate::utils::get_default_drive;

/// メインメニュー
pub struct Menu;

#[derive(Debug, Clone)]
pub enum MenuAction {
    ValidateStructure,
    ScanDrive,
    RecommendPlacement,
    InitStructure,
    ExecutePlacement,
    OrganizeRecords,
    Exit,
}

impl MenuAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            MenuAction::ValidateStructure => "フォルダ構造を検証",
            MenuAction::ScanDrive => "ドライブをスキャン",
            MenuAction::RecommendPlacement => "推奨配置を提案",
            MenuAction::InitStructure => "標準フォルダ構造を作成",
            MenuAction::ExecutePlacement => "推奨配置を実行",
            MenuAction::OrganizeRecords => "Recordフォルダを整理",
            MenuAction::Exit => "終了",
        }
    }
}

impl Menu {
    /// メインメニューを表示
    pub fn show_main_menu() -> Result<MenuAction> {
        let options = vec![
            MenuAction::ValidateStructure,
            MenuAction::ScanDrive,
            MenuAction::RecommendPlacement,
            MenuAction::InitStructure,
            MenuAction::ExecutePlacement,
            MenuAction::OrganizeRecords,
            MenuAction::Exit,
        ];

        let options_str: Vec<&str> = options.iter().map(|a| a.as_str()).collect();
        let options_str_clone = options_str.clone();

        let ans = Select::new("何を実行しますか？", options_str)
            .with_help_message("↑↓キーで選択、Enterで決定")
            .prompt()?;

        let index = options_str_clone.iter().position(|&x| x == ans).unwrap();
        Ok(options[index].clone())
    }

    /// ドライブを選択（デフォルトは実行ファイルのドライブ）
    pub fn select_drive() -> Result<PathBuf> {
        let default_drive = get_default_drive()?;
        
        let drive_str = format!("{} (デフォルト)", default_drive.display());
        let custom_str = "カスタムパスを指定";

        let ans = Select::new(
            "対象ドライブを選択してください",
            vec![&drive_str, custom_str],
        )
        .with_help_message("↑↓キーで選択、Enterで決定")
        .prompt()?;

        if ans == custom_str {
            let path = inquire::Text::new("パスを入力してください:")
                .with_default(&default_drive.to_string_lossy())
                .prompt()?;
            Ok(PathBuf::from(path))
        } else {
            Ok(default_drive)
        }
    }

    /// ドライラン確認
    pub fn confirm_dry_run() -> Result<bool> {
        Confirm::new("ドライラン（実際には実行しない）で実行しますか？")
            .with_default(true)
            .with_help_message("Yes: 確認のみ、No: 実際に実行")
            .prompt()
            .map_err(|e| anyhow::anyhow!("入力エラー: {}", e))
    }
}

