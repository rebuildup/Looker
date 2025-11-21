use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

/// ファイル管理CLIツールの引数定義
#[derive(Parser)]
#[command(name = "rfm")] // アプリ名
#[command(about = "Rust製ファイルマネージャー", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// ファイル一覧を表示
    #[command(visible_alias = "l")] // "l" でも呼び出せる
    Ls {
        /// 対象パス（指定がない場合はカレントディレクトリ）
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// 新しいディレクトリを作成
    Mkdir {
        /// 作成するディレクトリ名
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Ls { path } => {
            // フォルダの中身を読み取る
            let entries = fs::read_dir(path)
                .with_context(|| format!("ディレクトリの読み込みに失敗しました: {:?}", path))?;

            println!("path: {:?} の中身:", path);
            for entry in entries {
                let entry = entry?;
                let file_type = if entry.file_type()?.is_dir() { "DIR" } else { "FILE" };
                println!("[{}] {:?}", file_type, entry.file_name());
            }
        }
        Commands::Mkdir { path } => {
            fs::create_dir_all(path)
                .with_context(|| format!("ディレクトリの作成に失敗しました: {:?}", path))?;
            println!("作成完了: {:?}", path);
        }
    }

    Ok(())
}