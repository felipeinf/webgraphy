use crate::db::{
    archive_domain, archive_page, create_tag, delete_tag, export_json, get_domain_detail,
    get_graph_data, get_page_detail, get_sync_status, import_json, list_tags, set_domain_expanded,
    set_domain_tag, toggle_domain,
};
use crate::models::{
    DomainDetail, GraphData, ImportSummary, PageDetail, SyncStatus, SyncSummary, Tag,
};
use crate::sync;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: std::path::PathBuf,
}

#[tauri::command]
pub async fn get_favicon(
    state: State<'_, AppState>,
    hostname: String,
) -> Result<Option<String>, String> {
    crate::favicon::get_favicon_data_url(&state.data_dir, &hostname)
}

#[tauri::command]
pub async fn sync_tabs(state: State<'_, AppState>) -> Result<SyncSummary, String> {
    let collected = sync::collect_tabs();
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    sync::persist_tabs(&mut conn, collected)
}

#[tauri::command]
pub fn get_graph(
    state: State<'_, AppState>,
    search: Option<String>,
    expanded_domains: Option<Vec<i64>>,
    tag_ids: Option<Vec<i64>>,
) -> Result<GraphData, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    get_graph_data(
        &conn,
        search.as_deref(),
        expanded_domains.as_deref(),
        tag_ids.as_deref(),
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
pub fn archive_domain_cmd(state: State<'_, AppState>, domain_id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    archive_domain(&conn, domain_id, &now).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tags_cmd(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    list_tags(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag_cmd(state: State<'_, AppState>, name: String) -> Result<Tag, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Tag name is empty".to_string());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    create_tag(&conn, trimmed).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag_cmd(state: State<'_, AppState>, tag_id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    delete_tag(&conn, tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_domain_tag_cmd(
    state: State<'_, AppState>,
    domain_id: i64,
    tag_id: i64,
    assigned: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_domain_tag(&conn, domain_id, tag_id, assigned).map_err(|e| e.to_string())
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
pub async fn fetch_domain_meta_cmd(
    state: State<'_, AppState>,
    domain_id: i64,
) -> Result<DomainDetail, String> {
    let hostname = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let detail = get_domain_detail(&conn, domain_id).map_err(|e| e.to_string())?;
        if detail.meta_title.is_some() {
            return Ok(detail);
        }
        detail.hostname
    };

    let fetched = crate::domain_meta::fetch_domain_meta(&hostname);

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    match fetched {
        Some((title, description)) => {
            crate::db::set_domain_meta(&conn, domain_id, &title, &description)
                .map_err(|e| e.to_string())?;
        }
        None => {
            crate::db::set_domain_meta(&conn, domain_id, "", "").map_err(|e| e.to_string())?;
        }
    }
    get_domain_detail(&conn, domain_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_graph(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let content = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        export_json(&conn).map_err(|e| e.to_string())?
    };

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("webgraphy-export.json")
        .save_file(move |path| {
            let _ = tx.send(path);
        });

    let Some(file_path) = rx.recv().map_err(|e| e.to_string())? else {
        return Ok(None);
    };

    let path_buf = file_path.as_path().ok_or("Invalid path")?;
    std::fs::write(path_buf, content).map_err(|e| e.to_string())?;
    Ok(Some(path_buf.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn import_graph(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<ImportSummary>, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .pick_file(move |path| {
            let _ = tx.send(path);
        });

    let Some(file_path) = rx.recv().map_err(|e| e.to_string())? else {
        return Ok(None);
    };

    let path_buf = file_path.as_path().ok_or("Invalid path")?;
    let raw = std::fs::read_to_string(path_buf).map_err(|e| e.to_string())?;

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    let (domains_upserted, pages_upserted) = import_json(&mut conn, &raw)?;
    Ok(Some(ImportSummary {
        domains_upserted,
        pages_upserted,
    }))
}
