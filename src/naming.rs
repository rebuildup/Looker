use regex::Regex;

/// ファイル名の命名規則チェック
pub struct NamingRule;

impl NamingRule {
    /// recordファイル向けルール:
    /// YYYYMMDDHHMMSS_[screen-capture|screen-record|voice-record].[extension]
    pub fn check_record_naming(filename: &str) -> bool {
        let pattern = r"^\d{14}_(screen-capture|screen-record|voice-record)\.[^.]+$";
        let re = Regex::new(pattern).unwrap();
        re.is_match(filename)
    }
}
