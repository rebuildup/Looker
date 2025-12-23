use std::path::Path;
use std::time::Duration;

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::record_manager::{ActionType, RecordFileAction, RecordOrganizationPlan};

pub struct UI;

impl UI {
    pub fn print_title() {
        let banner = r#"
 _                     _             
| |                   | |            
| | ___   ___ ___  ___| | _____ _ __ 
| |/ _ \ / __/ _ \/ __| |/ / _ \ '__|
| | (_) | (_|  __/ (__|   <  __/ |   
|_|\___/ \___\___|\___|_|\_\___|_|   
"#;
        println!("{}", banner.bright_cyan());
        println!(
            "{}",
            "Folder Orchestrator for Record workspace".bright_black()
        );
        println!("{}", "引数なしで実行すると、インタラクティブメニューが表示されます。".bright_black());
        Self::separator();
    }

    pub fn separator() {
        println!(
            "{}",
            "──────────────────────────────────────────────".bright_black()
        );
    }

    pub fn section(title: &str) {
        println!("\n{}", format!("■ {}", title).bright_white().bold());
        Self::separator();
    }

    pub fn info(message: &str) {
        println!("{}", format!("ℹ {message}").bright_blue());
    }

    pub fn success(message: &str) {
        println!("{}", format!("✓ {message}").bright_green().bold());
    }

    pub fn warning(message: &str) {
        println!("{}", format!("⚠ {message}").bright_yellow());
    }

    #[allow(dead_code)]
    pub fn error(message: &str) {
        println!("{}", format!("✗ {message}").bright_red().bold());
    }

    pub fn loading(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(80));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message(message.to_string());
        pb
    }

    pub fn render_plan_summary(plan: &RecordOrganizationPlan, verbose: bool) {
        println!(
            "{} {}",
            "Recordフォルダ:".bright_cyan().bold(),
            plan.record_root.display()
        );
        println!(
            "{} {}",
            "作成が必要なフォルダ:".bright_cyan(),
            plan.required_folders.len()
        );
        if !plan.required_folders.is_empty() {
            let folders = plan
                .required_folders
                .iter()
                .map(|p| format!("📁 {}", Self::format_path(p)));
            Self::preview_lines(folders, verbose);
        }

        println!("{} {}", "ファイル操作数:".bright_cyan(), plan.actions.len());
        if !plan.actions.is_empty() {
            let ops = plan.actions.iter().map(|action| {
                format!(
                    "{} {}",
                    Self::action_icon(action),
                    Self::format_action(action)
                )
            });
            Self::preview_lines(ops, verbose);
        }
    }

    fn preview_lines<I>(lines: I, verbose: bool)
    where
        I: Iterator<Item = String>,
    {
        let limit = if verbose { usize::MAX } else { 10 };
        let mut buffer = Vec::new();
        let mut count = 0usize;

        for line in lines {
            if verbose {
                println!("{line}");
                count += 1;
                continue;
            }

            if count < limit {
                buffer.push(line);
            }
            count += 1;
        }

        if !verbose {
            for line in &buffer {
                println!("{line}");
            }
            if count > limit {
                println!(
                    "{}",
                    format!("  ...あと {} 件", count - limit).bright_black()
                );
            }
        }
    }

    fn format_path(path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    fn format_action(action: &RecordFileAction) -> String {
        match action.action_type {
            ActionType::Move => format!(
                "{} → {}",
                Self::format_path(&action.source),
                Self::format_path(&action.target)
            ),
            ActionType::Rename => format!(
                "{} → {}",
                Self::format_path(&action.source),
                Self::format_path(&action.target)
            ),
            ActionType::MoveToCorrectLocation => format!(
                "{} → {}",
                Self::format_path(&action.source),
                Self::format_path(&action.target)
            ),
        }
    }

    fn action_icon(action: &RecordFileAction) -> &'static str {
        match action.action_type {
            ActionType::Move => "⇢",
            ActionType::Rename => "✎",
            ActionType::MoveToCorrectLocation => "⤴",
        }
    }
}
