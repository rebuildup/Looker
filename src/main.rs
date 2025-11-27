mod naming;
mod record_manager;
mod scanner;

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use record_manager::{
    ActionType, RecordFileAction, RecordManager, RecordOptions, RecordOrganizationPlan, RecordType,
};

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
    print_plan_summary(&plan, args.verbose);

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

fn print_plan_summary(plan: &RecordOrganizationPlan, verbose: bool) {
    println!("Recordフォルダ: {}", plan.record_root.display());
    println!("作成が必要なフォルダ: {}", plan.required_folders.len());
    if !plan.required_folders.is_empty() {
        let lines = plan
            .required_folders
            .iter()
            .map(|p| format!("  - {}", display_path(p)));
        show_preview(lines, verbose);
    }

    println!("ファイル操作数: {}", plan.actions.len());
    if !plan.actions.is_empty() {
        let formatter = |action: &RecordFileAction| -> String {
            match action.action_type {
                ActionType::Move => format!(
                    "[MOVE] {} -> {}",
                    display_path(&action.source),
                    display_path(&action.target)
                ),
                ActionType::Rename => format!(
                    "[RENAME] {} -> {}",
                    display_path(&action.source),
                    display_path(&action.target)
                ),
                ActionType::MoveToCorrectLocation => format!(
                    "[RELOCATE] {} -> {}",
                    display_path(&action.source),
                    display_path(&action.target)
                ),
            }
        };

        let lines = plan.actions.iter().map(|a| format!("  - {}", formatter(a)));
        show_preview(lines, verbose);
    }
}

fn show_preview<I>(items: I, verbose: bool)
where
    I: Iterator<Item = String>,
{
    let limit = if verbose { usize::MAX } else { 10 };
    let mut count = 0usize;
    let mut buffer: Vec<String> = Vec::new();

    for item in items {
        if verbose {
            println!("{item}");
            count += 1;
            continue;
        }

        if count < limit {
            buffer.push(item);
        }
        count += 1;
    }

    if !verbose {
        for line in &buffer {
            println!("{line}");
        }
    }

    if !verbose && count > limit {
        println!("  ...あと {} 件", count - limit);
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N]: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let normalized = input.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "y" | "yes"))
}
