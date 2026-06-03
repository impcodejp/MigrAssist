use tauri::{AppHandle, Emitter};

use crate::types::{ComparisonConfig, ComparisonResult, ConfigData, ProgressPayload};
use crate::{comparison, config, csv_reader};

#[tauri::command]
pub fn load_csv(path: String, encoding: String) -> Result<Vec<String>, String> {
    if path.trim().is_empty() {
        return Err("ファイルパスが未入力です".to_string());
    }
    csv_reader::read_headers(&path, &encoding)
}

#[tauri::command]
pub async fn execute_comparison(
    app: AppHandle,
    config: ComparisonConfig,
) -> Result<ComparisonResult, String> {
    validate_config(&config)?;

    tokio::task::spawn_blocking(move || {
        let on_progress = |msg: &str, step: u32, total: u32| {
            let _ = app.emit(
                "comparison-progress",
                ProgressPayload {
                    message: msg.to_string(),
                    step,
                    total,
                },
            );
        };
        comparison::run(&config, on_progress)
    })
    .await
    .map_err(|e| format!("実行エラー: {}", e))?
}

#[tauri::command]
pub fn save_config(path: String, data: ConfigData) -> Result<(), String> {
    config::save(&path, &data)
}

#[tauri::command]
pub fn load_config(path: String) -> Result<ConfigData, String> {
    config::load(&path)
}

fn validate_config(config: &ComparisonConfig) -> Result<(), String> {
    if config.mjs_path.trim().is_empty() {
        return Err("MJSシステムのCSVファイルが指定されていません".to_string());
    }
    if config.other_path.trim().is_empty() {
        return Err("他社システムのCSVファイルが指定されていません".to_string());
    }
    if config.output_dir.trim().is_empty() {
        return Err("出力フォルダが指定されていません".to_string());
    }
    if config.columns.is_empty() {
        return Err("ファイルが取り込まれていません".to_string());
    }
    let key_col = &config.columns[0];
    if key_col.other_header.trim().is_empty() {
        return Err("キー列の他社システム側の列が選択されていません".to_string());
    }
    let compare_cols: Vec<_> = config.columns[1..].iter().filter(|c| c.is_compare).collect();
    if compare_cols.is_empty() {
        return Err("比較対象の列が1件も選択されていません".to_string());
    }
    for col in &compare_cols {
        if col.other_header.trim().is_empty() {
            return Err(format!(
                "比較対象列「{}」の他社システム側の列が選択されていません",
                col.mjs_header
            ));
        }
    }
    Ok(())
}
