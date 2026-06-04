use std::fs;

use crate::types::ConfigData;

pub fn save(path: &str, config: &ConfigData) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(config).map_err(|e| format!("設定シリアライズエラー: {}", e))?;
    fs::write(path, json).map_err(|e| format!("設定ファイル書き込みエラー: {}", e))
}

pub fn load(path: &str) -> Result<ConfigData, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("設定ファイル読み込みエラー: {}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("設定ファイル解析エラー: {}", e))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnMapping;

    fn test_config() -> ConfigData {
        ConfigData {
            columns: vec![
                ColumnMapping {
                    mjs_header: "社員コード".to_string(),
                    other_header: "EMP_CODE".to_string(),
                    is_compare: false,
                    tolerance: 0,
                },
                ColumnMapping {
                    mjs_header: "出勤日数".to_string(),
                    other_header: "WORK_DAYS".to_string(),
                    is_compare: true,
                    tolerance: 2,
                },
            ],
        }
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(name);
        p
    }

    #[test]
    fn save_and_load_roundtrip() {
        let config = test_config();
        let path = temp_path("migrassist_config_test.json");
        let path_str = path.to_str().unwrap();

        save(path_str, &config).unwrap();
        let loaded = load(path_str).unwrap();

        assert_eq!(loaded.columns.len(), 2);
        assert_eq!(loaded.columns[0].mjs_header, "社員コード");
        assert_eq!(loaded.columns[0].other_header, "EMP_CODE");
        assert!(!loaded.columns[0].is_compare);
        assert_eq!(loaded.columns[1].mjs_header, "出勤日数");
        assert_eq!(loaded.columns[1].tolerance, 2);
        assert!(loaded.columns[1].is_compare);

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn load_nonexistent_file_returns_error() {
        let result = load("/nonexistent/path/config.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("設定ファイル読み込みエラー"));
    }

    #[test]
    fn load_invalid_json_returns_error() {
        let path = temp_path("migrassist_invalid_json_test.json");
        std::fs::write(&path, "これはJSONではありません").unwrap();
        let result = load(path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("設定ファイル解析エラー"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn save_creates_valid_json_file() {
        let config = test_config();
        let path = temp_path("migrassist_json_format_test.json");
        save(path.to_str().unwrap(), &config).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // pretty-print された JSON なので { を含む
        assert!(content.contains('{'));
        assert!(content.contains("社員コード"));

        std::fs::remove_file(path).ok();
    }
}
