use crate::db::{
    archive_page, export_html, export_json, export_markdown, get_domain_detail, get_graph_data,
    get_page_detail, get_sync_status, set_domain_expanded, toggle_domain,
};
use crate::models::{DomainDetail, GraphData, PageDetail, SyncStatus, SyncSummary};
use crate::sync;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: std::path::PathBuf,
}

#[tauri::command]
pub fn get_favicon(state: State<'_, AppState>, hostname: String) -> Result<Option<String>, String> {
    crate::favicon::get_favicon_data_url(&state.data_dir, &hostname)
}

#[tauri::command]
pub fn sync_tabs(state: State<'_, AppState>) -> Result<SyncSummary, String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    sync::run_sync(&mut conn)
}

#[tauri::command]
pub fn get_graph(
    state: State<'_, AppState>,
    search: Option<String>,
    expanded_domains: Option<Vec<i64>>,
) -> Result<GraphData, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    get_graph_data(
        &conn,
        search.as_deref(),
        expanded_domains.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_domain_expanded_cmd(
    state: State<'_, AppState>,
    domain_id: i64,
    expanded: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_domain_expanded(&conn, domain_id, expanded).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_domain_expanded(state: State<'_, AppState>, domain_id: i64) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    toggle_domain(&conn, domain_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn archive_page_cmd(state: State<'_, AppState>, page_id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    archive_page(&conn, page_id, &now).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_status_cmd(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    get_sync_status(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_page_detail_cmd(state: State<'_, AppState>, page_id: i64) -> Result<PageDetail, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    get_page_detail(&conn, page_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_domain_detail_cmd(
    state: State<'_, AppState>,
    domain_id: i64,
) -> Result<DomainDetail, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    get_domain_detail(&conn, domain_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_graph(
    state: State<'_, AppState>,
    format: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    match format.as_str() {
        "json" => export_json(&conn).map_err(|e| e.to_string()),
        "markdown" => export_markdown(&conn).map_err(|e| e.to_string()),
        "html" => export_html(&conn).map_err(|e| e.to_string()),
        _ => Err(format!("Unknown export format: {format}")),
    }
}

#[tauri::command]
pub fn save_export(
    app: tauri::AppHandle,
    content: String,
    format: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let ext = match format.as_str() {
        "json" => "json",
        "markdown" => "md",
        "html" => "html",
        _ => "txt",
    };

    let path = app
        .dialog()
        .file()
        .add_filter("Export", &[ext])
        .set_file_name(&format!("webgraphy-export.{ext}"))
        .blocking_save_file();

    if let Some(file_path) = path {
        let path_buf = file_path.as_path().ok_or("Invalid path")?;
        std::fs::write(path_buf, content).map_err(|e| e.to_string())?;
        Ok(Some(path_buf.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}
