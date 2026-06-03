mod commands;
mod comparison;
mod config;
mod csv_reader;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_csv,
            commands::execute_comparison,
            commands::save_config,
            commands::load_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
