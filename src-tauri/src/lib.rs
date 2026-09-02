mod collectors;
mod commands;
mod db;
mod favicon;
mod models;
mod normalize;
mod snss_fallback;
mod sync;

use commands::AppState;
use db::open_db;
use std::path::PathBuf;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = app_data_dir();
    let db_path = data_dir.join("webgraphy.db");
    let conn = open_db(&db_path).expect("Failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(conn),
            data_dir,
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_favicon,
            commands::sync_tabs,
            commands::get_graph,
            commands::set_domain_expanded_cmd,
            commands::toggle_domain_expanded,
            commands::archive_page_cmd,
            commands::open_url,
            commands::get_sync_status_cmd,
            commands::get_page_detail_cmd,
            commands::get_domain_detail_cmd,
            commands::export_graph,
            commands::save_export,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn app_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("com.felipeinf.webgraphy")
}
