use regex::Regex;

/// ファイル名の命名規則チェック
pub struct NamingRule;

impl NamingRule {
    /// record ファイルの想定フォーマット:
    /// YYYYMMDDHHMMSS_[screen-capture|screen-record|voice-record][-N].[extension]
    pub fn check_record_naming(filename: &str) -> bool {
        // 末尾に -2, -3 ... のような重複回避用サフィックスが付くことを許容する
        let pattern =
            r"^\d{14}_(screen-capture|screen-record|voice-record)(-\d+)?\.[^.]+$";
        let re = Regex::new(pattern).unwrap();
        re.is_match(filename)
    }
}

