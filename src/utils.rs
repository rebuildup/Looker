use std::path::PathBuf;
use anyhow::{Context, Result};

/// デフォルトドライブを取得（実行ファイルが存在するドライブ）
pub fn get_default_drive() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()
        .context("実行ファイルのパスを取得できませんでした")?;
    
    // Windowsの場合、ドライブレターを取得
    #[cfg(windows)]
    {
        if let Some(prefix) = exe_path.components().next() {
            if let std::path::Component::Prefix(prefix_comp) = prefix {
                if let std::path::Prefix::Disk(drive_letter) = prefix_comp.kind() {
                    let drive = format!("{}:\\", drive_letter as char);
                    return Ok(PathBuf::from(drive));
                }
            }
        }
    }
    
    // フォールバック: 実行ファイルの親ディレクトリ
    exe_path.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("親ディレクトリを取得できませんでした"))
}

