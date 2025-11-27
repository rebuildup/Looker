mod menu;
mod naming;
mod record_manager;
mod scanner;
mod ui;

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use menu::{Menu, MenuAction};
use record_manager::{RecordManager, RecordOptions, RecordType};
use ui::UI;

#[derive(Parser, Debug)]
#[command(name = "looker")]
#[command(about = "Recordフォルダを実用的に整理するための軽量CLI")]
struct Cli {
    /// 0_inbox/record を含むルートディレクトリ
    #[arg(long, value_name = "PATH", default_value = ".")]
    root: PathBuf,

    /// Recordフォルダを直接指定したい場合はこちらを使用
    #[arg(long, value_name = "PATH")]
    record_path: Option<PathBuf>,

    /// 対象とするrecord種別（複数指定可）
    #[arg(long = "record-type", value_enum)]
    record_types: Vec<RecordKind>,

    /// screen/voiceなどの取り違いチェックを省略して高速化
    #[arg(long)]
    fast: bool,

    /// 計画された変更を実際に適用
    #[arg(long)]
    apply: bool,

    /// 確認なしで適用（--applyを暗黙的に有効化）
    #[arg(long, alias = "y")]
    yes: bool,

    /// すべての予定アクションとフォルダ作成を表示
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
    let mut record_root = args
        .record_path
        .unwrap_or_else(|| args.root.join("0_inbox").join("record"));
    if record_root.as_os_str().is_empty() {
        record_root = PathBuf::from(".");
    }

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
        println!("Recordフォルダは既に整っています。");
        return Ok(());
    }

    let apply_changes = args.apply || args.yes;
    if !apply_changes {
        println!("\n--apply を指定すると上記の変更を適用できます。");
        return Ok(());
    }

    if !args.yes && !confirm("変更を適用しますか?")? {
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
            MenuAction::AnalyzeOnly => handle_interactive_flow(false)?,
            MenuAction::OrganizeNow => handle_interactive_flow(true)?,
            MenuAction::Exit => {
                UI::info("終了します");
                break;
            }
        }
    }

    Ok(())
}

fn handle_interactive_flow(apply: bool) -> Result<()> {
    let record_root = Menu::ask_record_root()?;
    let options = Menu::ask_record_options()?;

    if apply {
        UI::section("Recordフォルダ整理（プレビュー＆実行）");
    } else {
        UI::section("Recordフォルダのプレビュー");
    }

    UI::info(&format!("対象: {}", record_root.display()));

    let spinner = UI::loading("フォルダを解析中...");
    let plan = RecordManager::plan(&record_root, &options)?;
    spinner.finish_and_clear();

    UI::render_plan_summary(&plan, false);

    if plan.is_empty() {
        UI::success("整理は不要です。");
        return Ok(());
    }

    if !apply {
        UI::info("適用は行わず、メニューに戻ります。");
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
