use std::env;
use std::fmt;
use std::path::PathBuf;

use anyhow::Result;
use inquire::{Confirm, MultiSelect, Select, Text};

use crate::record_manager::{RecordOptions, RecordType};

#[derive(Clone, Copy)]
pub enum MenuAction {
    AnalyzeOnly,
    OrganizeNow,
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

#[derive(Clone)]
struct RecordTypeChoice {
    label: &'static str,
    kind: RecordType,
}

impl fmt::Display for RecordTypeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl Menu {
    pub fn show_main_menu() -> Result<MenuAction> {
        let choices = vec![
            MenuChoice {
                label: "Recordフォルダの状況をプレビュー",
                action: MenuAction::AnalyzeOnly,
            },
            MenuChoice {
                label: "整理を実行（計画→適用）",
                action: MenuAction::OrganizeNow,
            },
            MenuChoice {
                label: "終了する",
                action: MenuAction::Exit,
            },
        ];

        let selected = Select::new("実行したいアクションを選択してください", choices)
            .prompt()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        Ok(selected.action)
    }

    pub fn ask_record_root() -> Result<PathBuf> {
        let default_path = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("0_inbox")
            .join("record");
        let default_str = default_path.to_string_lossy().to_string();

        let answer = Text::new("Recordフォルダのパスを入力してください")
            .with_default(default_str.as_str())
            .prompt()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let trimmed = answer.trim();
        if trimmed.is_empty() {
            Ok(PathBuf::from(default_str))
        } else {
            Ok(PathBuf::from(trimmed))
        }
    }

    pub fn ask_record_options() -> Result<RecordOptions> {
        let mut options = RecordOptions::default();

        let use_filter = Confirm::new("対象のRecord種別を絞り込みますか？")
            .with_default(false)
            .prompt()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        if use_filter {
            let selected = MultiSelect::new(
                "対象にするRecord種別を選択（スペースで切り替え）",
                Self::record_type_choices(),
            )
            .prompt()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

            if !selected.is_empty() {
                options.target_types = selected.iter().map(|choice| choice.kind.clone()).collect();
            }
        }

        let enable_check = Confirm::new("取り違え・命名規則チェックを実行しますか？")
            .with_default(true)
            .prompt()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        options.check_misplaced = enable_check;

        Ok(options)
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

    fn record_type_choices() -> Vec<RecordTypeChoice> {
        vec![
            RecordTypeChoice {
                label: "Screen Capture",
                kind: RecordType::ScreenCapture,
            },
            RecordTypeChoice {
                label: "Screen Record",
                kind: RecordType::ScreenRecord,
            },
            RecordTypeChoice {
                label: "Voice Record",
                kind: RecordType::VoiceRecord,
            },
        ]
    }
}
