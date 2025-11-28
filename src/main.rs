mod menu;
mod naming;
mod record_manager;
mod scanner;
mod ui;

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};
use menu::{Menu, MenuAction};
use record_manager::{RecordManager, RecordOptions, RecordType};
use ui::UI;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "looker")]
#[command(about = "Recordフォルダを安全に整理するための小さな CLI ツール")]
struct Cli {
    /// ルートディレクトリ（record を探す起点）※現在は未使用
    #[arg(long, value_name = "PATH", default_value = ".")]
    root: PathBuf,

    /// Record フォルダを直接指定したい場合に使用（通常は未使用）
    #[arg(long, value_name = "PATH")]
    record_path: Option<PathBuf>,

    /// 対象とする record 種別（通常は全て）
    #[arg(long = "record-type", value_enum)]
    record_types: Vec<RecordKind>,

    /// screen/voice などの誤配置チェックを省略する（高速モード）
    #[arg(long)]
    fast: bool,

    /// プランされた変更を自動的に適用する
    #[arg(long)]
    apply: bool,

    /// 確認無しで適用する（--apply を前提にする）
    #[arg(long, alias = "y")]
    yes: bool,

    /// すべての詳細を表示する
    #[arg(long)]
    verbose: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum RecordKind {
    ScreenCapture,
    ScreenRecord,
    VoiceRecord,
}

impl From<RecordKind> for RecordType {
    fn from(kind: RecordKind) -> Self {
        match kind {
            RecordKind::ScreenCapture => RecordType::ScreenCapture,
            RecordKind::ScreenRecord => RecordType::ScreenRecord,
            RecordKind::VoiceRecord => RecordType::VoiceRecord,
        }
    }
}

fn main() -> Result<()> {
    if std::env::args().len() == 1 {
        return run_interactive_mode();
    }
    run_cli_mode()
}

fn run_cli_mode() -> Result<()> {
    let args = Cli::parse();

    // 将来の拡張余地を残しつつ、デフォルトでは auto detect する
    let record_root = if let Some(path) = args.record_path {
        path
    } else {
        auto_detect_record_root()?
    };

    let mut options = RecordOptions {
        check_misplaced: !args.fast,
        ..RecordOptions::default()
    };
    if !args.record_types.is_empty() {
        options
            .target_types
            .extend(args.record_types.iter().map(|kind| RecordType::from(*kind)));
    }

    let plan = RecordManager::plan(&record_root, &options)?;
    UI::render_plan_summary(&plan, args.verbose);

    if plan.is_empty() {
        println!("Recordフォルダは既に整理済みです。");
        return Ok(());
    }

    let apply_changes = args.apply || args.yes;
    if !apply_changes {
        println!("\n--apply を付けると、上記の変更を適用します。");
        return Ok(());
    }

    if !args.yes && !confirm("変更を適用しますか？")? {
        println!("適用をキャンセルしました。");
        return Ok(());
    }

    RecordManager::apply(&plan)?;
    println!(
        "フォルダ作成 {} 件 / ファイル操作 {} 件を適用しました。",
        plan.required_folders.len(),
        plan.actions.len()
    );

    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N]: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let normalized = input.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "y" | "yes"))
}

fn run_interactive_mode() -> Result<()> {
    UI::print_title();

    loop {
        match Menu::show_main_menu()? {
            MenuAction::OrganizeNow => handle_interactive_flow()?,
            MenuAction::Exit => {
                UI::info("終了します。");
                break;
            }
        }
    }

    Ok(())
}

fn handle_interactive_flow() -> Result<()> {
    let record_root = auto_detect_record_root()?;
    let options = Menu::ask_record_options()?;

    UI::section("Recordフォルダの整理");
    UI::info(&format!("対象: {}", record_root.display()));

    let spinner = UI::loading("フォルダ構造を解析中...");
    let plan = RecordManager::plan(&record_root, &options)?;
    spinner.finish_and_clear();

    UI::render_plan_summary(&plan, false);

    if plan.is_empty() {
        UI::success("変更は不要です。");
        return Ok(());
    }

    if Menu::confirm_execution(plan.actions.len())? {
        let spinner = UI::loading("変更を適用中...");
        RecordManager::apply(&plan)?;
        spinner.finish_and_clear();
        UI::success("すべての変更を適用しました。");
    } else {
        UI::warning("適用をキャンセルしました。");
    }

    Ok(())
}

/// 現在のドライブのルートから辿って record フォルダを検出する
fn auto_detect_record_root() -> Result<PathBuf> {
    let current = std::env::current_dir()?;

    // ドライブルートまで遡る（Windows / 他 OS 両対応）
    let mut root = current.clone();
    while let Some(parent) = root.parent() {
        root = parent.to_path_buf();
    }

    // ルート配下から record ディレクトリを検索する（深さは適度に制限）
    for entry in WalkDir::new(&root)
        .follow_links(false)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir()
            && entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("record")
        {
            return Ok(entry.into_path());
        }
    }

    Err(anyhow!(
        "record フォルダが見つかりませんでした（開始パス: {:?}）",
        root
    ))
}

