mod structure;
mod scanner;
mod naming;
mod recommender;
mod executor;
mod record_manager;
mod ui;
mod menu;
mod utils;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

use structure::FolderStructure;
use scanner::DriveScanner;
use recommender::PlacementRecommender;
use executor::Executor;
use record_manager::RecordManager;
use menu::{Menu, MenuAction};
use ui::UI;

#[derive(Parser)]
#[command(name = "looker")]
#[command(about = "外付けSSDのフォルダ構造を最適化するCLIアプリケーション", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// フォルダ構造を検証
    Validate {
        /// ルートディレクトリ
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    /// ドライブ全体をスキャン
    Scan {
        /// スキャンするパス
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 推奨配置を提案
    Recommend {
        /// スキャンするパス
        #[arg(default_value = ".")]
        path: PathBuf,
        /// ルートディレクトリ（推奨先の基準）
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    /// 標準フォルダ構造を作成
    Init {
        /// ルートディレクトリ
        #[arg(default_value = ".")]
        root: PathBuf,
        /// ドライラン（実際には実行しない）
        #[arg(long)]
        dry_run: bool,
    },
    /// 推奨配置を実行
    Execute {
        /// スキャンするパス
        #[arg(default_value = ".")]
        path: PathBuf,
        /// ルートディレクトリ
        #[arg(default_value = ".")]
        root: PathBuf,
        /// ドライラン（実際には実行しない）
        #[arg(long)]
        dry_run: bool,
    },
    /// 命名規則をチェック
    CheckNaming {
        /// チェックするパス
        path: PathBuf,
        /// recordフォルダ内のファイルかどうか
        #[arg(long)]
        is_record: bool,
    },
}

fn main() -> Result<()> {
    // 引数が指定されている場合は従来のCLIモード
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        return run_cli_mode();
    }

    // 引数がない場合は対話型メニュー
    run_interactive_mode()
}

fn run_cli_mode() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Validate { root } => {
            let result = FolderStructure::validate_structure(root)?;
            result.print_report();
        }
        Commands::Scan { path } => {
            println!("スキャン中: {:?}", path);
            let files = DriveScanner::scan(path)?;
            let categories = DriveScanner::categorize_files(&files);
            categories.print_summary();
        }
        Commands::Recommend { path, root } => {
            println!("スキャン中: {:?}", path);
            let files = DriveScanner::scan(path)?;
            let file_infos: Vec<_> = files.iter()
                .filter(|f| !f.is_dir)
                .cloned()
                .collect();

            println!("\n推奨配置を生成中...");
            let recommendations = PlacementRecommender::recommend_batch(&file_infos, root);

            println!("\n=== 推奨配置 ===");
            for rec in &recommendations {
                rec.print();
            }
        }
        Commands::Init { root, dry_run } => {
            let executor = Executor::new(*dry_run);
            if *dry_run {
                println!("[DRY RUN] 標準フォルダ構造を作成します");
            } else {
                println!("標準フォルダ構造を作成します");
            }
            executor.create_standard_structure(root)?;
            println!("完了しました");
        }
        Commands::Execute { path, root, dry_run } => {
            let executor = Executor::new(*dry_run);
            
            if *dry_run {
                println!("[DRY RUN] 推奨配置を実行します");
            } else {
                println!("推奨配置を実行します");
            }

            println!("スキャン中: {:?}", path);
            let files = DriveScanner::scan(path)?;
            let file_infos: Vec<_> = files.iter()
                .filter(|f| !f.is_dir)
                .cloned()
                .collect();

            let recommendations = PlacementRecommender::recommend_batch(&file_infos, root);
            let result = executor.execute_recommendations(&recommendations)?;
            result.print_report();
        }
        Commands::CheckNaming { path, is_record } => {
            let validation = naming::NamingRule::validate_filename(path, *is_record);
            validation.print_report();
        }
    }

    Ok(())
}

fn run_interactive_mode() -> Result<()> {
    UI::print_title();

    loop {
        let action = Menu::show_main_menu()?;

        match action {
            MenuAction::ValidateStructure => {
                UI::section("フォルダ構造の検証");
                let root = Menu::select_drive()?;
                UI::info(&format!("検証中: {}", root.display()));
                
                let pb = UI::loading("検証中...");
                let result = FolderStructure::validate_structure(&root)?;
                pb.finish_and_clear();
                
                result.print_report();
                UI::separator();
            }
            MenuAction::ScanDrive => {
                UI::section("ドライブのスキャン");
                let path = Menu::select_drive()?;
                UI::info(&format!("スキャン中: {}", path.display()));
                
                let pb = UI::loading("スキャン中...");
                let files = DriveScanner::scan(&path)?;
                let categories = DriveScanner::categorize_files(&files);
                pb.finish_and_clear();
                
                categories.print_summary();
                UI::separator();
            }
            MenuAction::RecommendPlacement => {
                UI::section("推奨配置の提案");
                let root = Menu::select_drive()?;
                UI::info(&format!("スキャン中: {}", root.display()));
                
                let pb = UI::loading("スキャン中...");
                let files = DriveScanner::scan(&root)?;
                let file_infos: Vec<_> = files.iter()
                    .filter(|f| !f.is_dir)
                    .cloned()
                    .collect();
                pb.finish_and_clear();
                
                let pb = UI::loading("推奨配置を生成中...");
                let recommendations = PlacementRecommender::recommend_batch(&file_infos, &root);
                pb.finish_and_clear();
                
                UI::section("推奨配置");
                for rec in &recommendations {
                    rec.print();
                }
                UI::separator();
            }
            MenuAction::InitStructure => {
                UI::section("標準フォルダ構造の作成");
                let root = Menu::select_drive()?;
                let dry_run = Menu::confirm_dry_run()?;
                
                let executor = Executor::new(dry_run);
                if dry_run {
                    UI::warning("ドライランモード: 実際には作成しません");
                }
                
                let pb = UI::loading("フォルダ構造を作成中...");
                executor.create_standard_structure(&root)?;
                pb.finish_and_clear();
                
                UI::success("標準フォルダ構造の作成が完了しました");
                UI::separator();
            }
            MenuAction::ExecutePlacement => {
                UI::section("推奨配置の実行");
                let root = Menu::select_drive()?;
                let dry_run = Menu::confirm_dry_run()?;
                
                let executor = Executor::new(dry_run);
                if dry_run {
                    UI::warning("ドライランモード: 実際には実行しません");
                }
                
                UI::info(&format!("スキャン中: {}", root.display()));
                let pb = UI::loading("スキャン中...");
                let files = DriveScanner::scan(&root)?;
                let file_infos: Vec<_> = files.iter()
                    .filter(|f| !f.is_dir)
                    .cloned()
                    .collect();
                pb.finish_and_clear();
                
                let pb = UI::loading("推奨配置を生成中...");
                let recommendations = PlacementRecommender::recommend_batch(&file_infos, &root);
                pb.finish_and_clear();
                
                let pb = UI::loading("ファイルを移動中...");
                let result = executor.execute_recommendations(&recommendations)?;
                pb.finish_and_clear();
                
                result.print_report();
                UI::separator();
            }
            MenuAction::OrganizeRecords => {
                UI::section("Recordフォルダの整理");
                let root = Menu::select_drive()?;
                let dry_run = Menu::confirm_dry_run()?;
                
                if dry_run {
                    UI::warning("ドライランモード: 実際には実行しません");
                }
                
                let pb = UI::loading("Recordフォルダを整理中...");
                let result = RecordManager::organize_records(&root, dry_run)?;
                pb.finish_and_clear();
                
                result.print_report();
                
                if !result.actions.is_empty() && !dry_run {
                    let confirm = inquire::Confirm::new("整理を実行しますか？")
                        .with_default(true)
                        .prompt()?;
                    
                    if confirm {
                        let pb = UI::loading("実行中...");
                        RecordManager::execute_actions(&result.actions, dry_run)?;
                        pb.finish_and_clear();
                        UI::success("Recordフォルダの整理が完了しました");
                    }
                }
                UI::separator();
            }
            MenuAction::Exit => {
                UI::info("終了します");
                break;
            }
        }
    }

    Ok(())
}