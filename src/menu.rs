use std::fmt;

use anyhow::Result;
use inquire::{Confirm, Select};

use crate::record_manager::RecordOptions;

#[derive(Clone, Copy)]
pub enum MenuAction {
    OrganizeNow,
    CreateGalleryShortcuts,
    EnsureStructure,
    Exit,
}

pub struct Menu;

#[derive(Clone)]
struct MenuChoice {
    label: &'static str,
    action: MenuAction,
}

impl fmt::Display for MenuChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl Menu {
    pub fn show_main_menu() -> Result<MenuAction> {
        let choices = vec![
            MenuChoice {
                label: "Recordフォルダを整理（プレビュー後に適用）",
                action: MenuAction::OrganizeNow,
            },
            MenuChoice {
                label: "プロジェクト成果物のショートカットを作成",
                action: MenuAction::CreateGalleryShortcuts,
            },
            MenuChoice {
                label: "標準フォルダ構造を確認・作成",
                action: MenuAction::EnsureStructure,
            },
            MenuChoice {
                label: "終了する",
                action: MenuAction::Exit,
            },
        ];

        let selected =
            Select::new("実行したいアクションを選択してください", choices)
                .prompt()
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        Ok(selected.action)
    }

    /// 余計な選択肢は廃止し、デフォルト設定のみ使用する
    pub fn ask_record_options() -> Result<RecordOptions> {
        Ok(RecordOptions::default())
    }

    pub fn confirm_execution(action_count: usize) -> Result<bool> {
        let message = if action_count == 0 {
            "変更を適用しますか？"
        } else {
            "上記の変更を適用しますか？"
        };

        Confirm::new(message)
            .with_default(true)
            .prompt()
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}

