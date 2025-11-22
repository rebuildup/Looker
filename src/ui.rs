use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// リッチなCLI出力
pub struct UI;

impl UI {
    /// タイトルを表示
    pub fn print_title() {
        println!("\n{}", "╔═══════════════════════════════════════╗".bright_cyan());
        println!("{}", "║         Looker - フォルダ管理        ║".bright_cyan().bold());
        println!("{}", "╚═══════════════════════════════════════╝".bright_cyan());
        println!();
    }

    /// 成功メッセージ
    pub fn success(message: &str) {
        println!("{} {}", "✓".bright_green().bold(), message.bright_green());
    }

    /// エラーメッセージ
    #[allow(dead_code)]
    pub fn error(message: &str) {
        println!("{} {}", "✗".bright_red().bold(), message.bright_red());
    }

    /// 警告メッセージ
    pub fn warning(message: &str) {
        println!("{} {}", "⚠".bright_yellow().bold(), message.bright_yellow());
    }

    /// 情報メッセージ
    pub fn info(message: &str) {
        println!("{} {}", "ℹ".bright_blue().bold(), message.bright_blue());
    }

    /// セクションタイトル
    pub fn section(title: &str) {
        println!("\n{} {}", "▶".bright_cyan().bold(), title.bright_cyan().bold());
        println!("{}", "─".repeat(50).bright_black());
    }

    /// プログレスバーを作成
    #[allow(dead_code)]
    pub fn create_progress_bar(len: u64) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    }

    /// スピナー付きローディングメッセージ
    pub fn loading(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    /// 区切り線
    pub fn separator() {
        println!("{}", "─".repeat(50).bright_black());
    }

    /// リストアイテム
    #[allow(dead_code)]
    pub fn list_item(index: usize, text: &str) {
        println!("  {} {}", format!("{}.", index).bright_cyan(), text);
    }

    /// キーと値のペア
    #[allow(dead_code)]
    pub fn key_value(key: &str, value: &str) {
        println!("  {}: {}", key.bright_white().bold(), value.bright_white());
    }
}

