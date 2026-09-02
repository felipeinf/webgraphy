use crate::collectors;
use crate::db::{
    cleanup_old_snapshots, finish_sync_run, insert_tab_snapshot, is_dismissed, refresh_domain_counts,
    start_sync_run, upsert_domain, upsert_page,
};
use crate::models::{CollectedTab, SyncSummary};
use crate::normalize::{extract_hostname, normalize_url};
use crate::snss_fallback;
use chrono::Utc;
use rusqlite::Connection;
use std::collections::HashSet;

pub struct CollectedTabs {
    pub tabs: Vec<CollectedTab>,
    pub errors: Vec<String>,
}

pub fn collect_tabs() -> CollectedTabs {
    let (mut tabs, mut errors) = collectors::collect_all();

    let browsers_found: HashSet<&str> = tabs.iter().map(|t| t.browser.as_str()).collect();
    let needs_snss = !browsers_found.contains("Chrome") || !browsers_found.contains("Opera");

    if needs_snss || tabs.is_empty() {
        let (snss_tabs, snss_errors) = snss_fallback::collect_snss_fallback();
        let existing_urls: HashSet<String> = tabs.iter().map(|t| t.url.clone()).collect();
        for tab in snss_tabs {
            if !existing_urls.contains(&tab.url) {
                tabs.push(tab);
            }
        }
        errors.extend(snss_errors);
    }

    CollectedTabs { tabs, errors }
}

pub fn persist_tabs(conn: &mut Connection, collected: CollectedTabs) -> Result<SyncSummary, String> {
    let CollectedTabs { tabs, errors } = collected;
    let now = Utc::now().to_rfc3339();

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let sync_id = start_sync_run(&tx, &now).map_err(|e| e.to_string())?;

    let mut pages_upserted = 0usize;
    let mut browsers_scanned: HashSet<String> = HashSet::new();

    for tab in &tabs {
        browsers_scanned.insert(tab.browser.clone());

        let normalized = match normalize_url(&tab.url) {
            Some(n) => n,
            None => continue,
        };

        if is_dismissed(&tx, &normalized).unwrap_or(false) {
            continue;
        }

        let hostname = match extract_hostname(&normalized) {
            Some(h) => h,
            None => continue,
        };

        let domain_key = crate::normalize::registrable_domain(&hostname);

        let domain_id = upsert_domain(&tx, &domain_key, &now).map_err(|e| e.to_string())?;
        let page_id = upsert_page(
            &tx,
            domain_id,
            &normalized,
            &tab.url,
            &tab.title,
            &now,
        )
        .map_err(|e| e.to_string())?;

        insert_tab_snapshot(
            &tx,
            page_id,
            &tab.browser,
            tab.window_id,
            tab.tab_index,
            sync_id,
            &now,
        )
        .map_err(|e| e.to_string())?;

        pages_upserted += 1;
    }

    refresh_domain_counts(&tx).map_err(|e| e.to_string())?;
    cleanup_old_snapshots(&tx, sync_id).map_err(|e| e.to_string())?;

    let browsers_vec: Vec<String> = browsers_scanned.into_iter().collect();
    let browsers_json =
        serde_json::to_string(&browsers_vec).unwrap_or_else(|_| "[]".to_string());

    finish_sync_run(&tx, sync_id, tabs.len() as i64, &browsers_json, &now)
        .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(SyncSummary {
        sync_id,
        tabs_found: tabs.len(),
        pages_upserted,
        browsers_scanned: browsers_vec,
        errors,
    })
}