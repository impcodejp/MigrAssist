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
