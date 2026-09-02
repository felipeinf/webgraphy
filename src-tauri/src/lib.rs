mod collectors;
mod commands;
mod db;
mod domain_meta;
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
            commands::archive_domain_cmd,
            commands::list_tags_cmd,
            commands::create_tag_cmd,
            commands::delete_tag_cmd,
            commands::set_domain_tag_cmd,
            commands::open_url,
            commands::get_sync_status_cmd,
            commands::get_page_detail_cmd,
            commands::get_domain_detail_cmd,
            commands::fetch_domain_meta_cmd,
            commands::export_graph,
            commands::import_graph,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn app_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(&home).join(".webgraphy");
    std::fs::create_dir_all(&dir).ok();
    migrate_legacy_data(&home, &dir);
    dir
}

fn migrate_legacy_data(home: &str, dest: &PathBuf) {
    let dest_db = dest.join("webgraphy.db");
    if dest_db.exists() {
        return;
    }

    let legacy = PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("com.felipeinf.webgraphy");
    let legacy_db = legacy.join("webgraphy.db");
    if legacy_db.exists() {
        let _ = std::fs::copy(&legacy_db, &dest_db);
    }

    let legacy_favicons = legacy.join("favicons");
    let dest_favicons = dest.join("favicons");
    if legacy_favicons.is_dir() && !dest_favicons.exists() {
        let _ = copy_dir_all(&legacy_favicons, &dest_favicons);
    }
}

fn copy_dir_all(src: &PathBuf, dest: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
