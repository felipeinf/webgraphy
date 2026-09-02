use crate::models::{
    DomainDetail, ExportDomain, ExportPage, ExportTree, GraphData, GraphLink, GraphNode, PageDetail,
    SyncStatus, Tag,
};
use rusqlite::{params, Connection, Result as SqlResult};
use url::Url;

pub fn is_dismissed(conn: &Connection, normalized_url: &str) -> SqlResult<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM dismissed_urls WHERE normalized_url = ?1",
        [normalized_url],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn upsert_domain(conn: &Connection, hostname: &str, now: &str) -> SqlResult<i64> {
    let favicon = format!(
        "https://www.google.com/s2/favicons?domain={}&sz=32",
        hostname
    );
    conn.execute(
        "INSERT INTO domains (hostname, favicon_url, last_seen_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(hostname) DO UPDATE SET last_seen_at = excluded.last_seen_at",
        params![hostname, favicon, now],
    )?;
    conn.query_row(
        "SELECT id FROM domains WHERE hostname = ?1",
        [hostname],
        |row| row.get(0),
    )
}

pub fn upsert_page(
    conn: &Connection,
    domain_id: i64,
    normalized_url: &str,
    original_url: &str,
    title: &str,
    now: &str,
) -> SqlResult<i64> {
    conn.execute(
        "INSERT INTO pages (domain_id, normalized_url, original_url, title, first_seen_at, last_seen_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT(normalized_url) DO UPDATE SET
           title = CASE WHEN excluded.title != '' AND excluded.title != 'New Tab' THEN excluded.title ELSE pages.title END,
           original_url = excluded.original_url,
           last_seen_at = excluded.last_seen_at,
           is_archived = 0",
        params![domain_id, normalized_url, original_url, title, now],
    )?;
    conn.query_row(
        "SELECT id FROM pages WHERE normalized_url = ?1",
        [normalized_url],
        |row| row.get(0),
    )
}

pub fn insert_tab_snapshot(
    conn: &Connection,
    page_id: i64,
    browser: &str,
    window_id: i64,
    tab_index: i64,
    sync_id: i64,
    now: &str,
) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO tab_snapshots (page_id, browser, window_id, tab_index, seen_at, sync_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![page_id, browser, window_id, tab_index, now, sync_id],
    )?;
    Ok(())
}

pub fn start_sync_run(conn: &Connection, now: &str) -> SqlResult<i64> {
    conn.execute(
        "INSERT INTO sync_runs (started_at) VALUES (?1)",
        [now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn finish_sync_run(
    conn: &Connection,
    sync_id: i64,
    tabs_found: i64,
    browsers: &str,
    now: &str,
) -> SqlResult<()> {
    conn.execute(
        "UPDATE sync_runs SET finished_at = ?1, tabs_found = ?2, browsers_scanned = ?3 WHERE id = ?4",
        params![now, tabs_found, browsers, sync_id],
    )?;
    Ok(())
}

pub fn refresh_domain_counts(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "UPDATE domains SET page_count = (
            SELECT COUNT(*) FROM pages
            WHERE pages.domain_id = domains.id AND pages.is_archived = 0
         );",
    )?;
    Ok(())
}

pub fn cleanup_old_snapshots(conn: &Connection, keep_sync_id: i64) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM tab_snapshots WHERE sync_id != ?1",
        [keep_sync_id],
    )?;
    Ok(())
}

pub fn set_domain_expanded(conn: &Connection, domain_id: i64, expanded: bool) -> SqlResult<()> {
    conn.execute(
        "UPDATE domains SET is_expanded = ?1 WHERE id = ?2",
        params![if expanded { 1 } else { 0 }, domain_id],
    )?;
    Ok(())
}

pub fn toggle_domain(conn: &Connection, domain_id: i64) -> SqlResult<bool> {
    let current: i64 = conn.query_row(
        "SELECT is_expanded FROM domains WHERE id = ?1",
        [domain_id],
        |row| row.get(0),
    )?;
    let new_val = if current == 0 { 1 } else { 0 };
    conn.execute(
        "UPDATE domains SET is_expanded = ?1 WHERE id = ?2",
        params![new_val, domain_id],
    )?;
    Ok(new_val == 1)
}

pub fn archive_page(conn: &Connection, page_id: i64, now: &str) -> SqlResult<()> {
    let normalized: String = conn.query_row(
        "SELECT normalized_url FROM pages WHERE id = ?1",
        [page_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "UPDATE pages SET is_archived = 1 WHERE id = ?1",
        [page_id],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO dismissed_urls (normalized_url, dismissed_at) VALUES (?1, ?2)",
        params![normalized, now],
    )?;
    refresh_domain_counts(conn)?;
    Ok(())
}

pub fn archive_domain(conn: &Connection, domain_id: i64, now: &str) -> SqlResult<()> {
    let mut stmt = conn.prepare(
        "SELECT id FROM pages WHERE domain_id = ?1 AND is_archived = 0",
    )?;
    let page_ids: Vec<i64> = stmt
        .query_map([domain_id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    for page_id in page_ids {
        archive_page(conn, page_id, now)?;
    }
    Ok(())
}

pub fn list_tags(conn: &Connection) -> SqlResult<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name FROM tags ORDER BY name COLLATE NOCASE")?;
    let rows = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    rows.collect()
}

pub fn list_tags_for_domain(conn: &Connection, domain_id: i64) -> SqlResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name FROM tags t
         JOIN domain_tags dt ON dt.tag_id = t.id
         WHERE dt.domain_id = ?1
         ORDER BY t.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([domain_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    rows.collect()
}

pub fn create_tag(conn: &Connection, name: &str) -> SqlResult<Tag> {
    let trimmed = name.trim();
    conn.execute(
        "INSERT INTO tags (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
        [trimmed],
    )?;
    conn.query_row(
        "SELECT id, name FROM tags WHERE name = ?1 COLLATE NOCASE",
        [trimmed],
        |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        },
    )
}

pub fn delete_tag(conn: &Connection, tag_id: i64) -> SqlResult<()> {
    conn.execute("DELETE FROM tags WHERE id = ?1", [tag_id])?;
    Ok(())
}

pub fn set_domain_tag(
    conn: &Connection,
    domain_id: i64,
    tag_id: i64,
    assigned: bool,
) -> SqlResult<()> {
    if assigned {
        conn.execute(
            "INSERT OR IGNORE INTO domain_tags (domain_id, tag_id) VALUES (?1, ?2)",
            params![domain_id, tag_id],
        )?;
    } else {
        conn.execute(
            "DELETE FROM domain_tags WHERE domain_id = ?1 AND tag_id = ?2",
            params![domain_id, tag_id],
        )?;
    }
    Ok(())
}

fn domain_matches_tags(conn: &Connection, domain_id: i64, tag_ids: &[i64]) -> SqlResult<bool> {
    if tag_ids.is_empty() {
        return Ok(true);
    }
    for tag_id in tag_ids {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM domain_tags WHERE domain_id = ?1 AND tag_id = ?2",
            params![domain_id, tag_id],
            |row| row.get(0),
        )?;
        if count > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn get_browsers_for_page(conn: &Connection, page_id: i64) -> SqlResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT browser FROM tab_snapshots WHERE page_id = ?1 ORDER BY browser",
    )?;
    let rows = stmt.query_map([page_id], |row| row.get(0))?;
    rows.collect()
}

pub fn get_sync_status(conn: &Connection) -> SqlResult<SyncStatus> {
    let last_sync: Option<String> = conn
        .query_row(
            "SELECT finished_at FROM sync_runs WHERE finished_at IS NOT NULL ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    let total_pages: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pages WHERE is_archived = 0",
        [],
        |row| row.get(0),
    )?;

    let total_domains: i64 = conn.query_row(
        "SELECT COUNT(*) FROM domains WHERE page_count > 0",
        [],
        |row| row.get(0),
    )?;

    let open_instances: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tab_snapshots",
        [],
        |row| row.get(0),
    )?;

    Ok(SyncStatus {
        last_sync_at: last_sync,
        total_pages,
        total_domains,
        open_instances,
    })
}

pub fn get_graph_data(
    conn: &Connection,
    search: Option<&str>,
    expanded_domains: Option<&[i64]>,
    tag_ids: Option<&[i64]>,
) -> SqlResult<GraphData> {
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    let search_filter = search.map(|s| s.to_lowercase());

    let mut domain_stmt = conn.prepare(
        "SELECT id, hostname, favicon_url, is_expanded, page_count
         FROM domains WHERE page_count > 0 ORDER BY page_count DESC, hostname ASC",
    )?;

    let domain_rows = domain_stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;

    for domain in domain_rows {
        let (id, hostname, favicon_url, _is_expanded, page_count) = domain?;

        if let Some(ids) = tag_ids {
            if !ids.is_empty() && !domain_matches_tags(conn, id, ids)? {
                continue;
            }
        }

        if let Some(ref filter) = search_filter {
            if !hostname.to_lowercase().contains(filter) {
                let has_match: bool = conn.query_row(
                    "SELECT COUNT(*) > 0 FROM pages
                     WHERE domain_id = ?1 AND is_archived = 0
                     AND (LOWER(title) LIKE ?2 OR LOWER(normalized_url) LIKE ?2)",
                    params![id, format!("%{filter}%")],
                    |row| row.get(0),
                )?;
                if !has_match {
                    continue;
                }
            }
        }

        let domain_node_id = format!("domain-{id}");
        nodes.push(GraphNode {
            id: domain_node_id.clone(),
            node_type: "domain".to_string(),
            label: format!("{hostname} ({page_count})"),
            hostname: Some(hostname.clone()),
            url: None,
            title: None,
            page_count: Some(page_count),
            browsers: vec![],
            favicon_url: favicon_url.clone(),
            domain_id: Some(id),
        });

        let should_expand = expanded_domains
            .map(|ids| ids.contains(&id))
            .unwrap_or(false);
        if !should_expand {
            continue;
        }

        let mut page_stmt = conn.prepare(
            "SELECT p.id, p.normalized_url, p.title, d.favicon_url
             FROM pages p
             JOIN domains d ON d.id = p.domain_id
             WHERE p.domain_id = ?1 AND p.is_archived = 0
             ORDER BY p.last_seen_at DESC",
        )?;

        let page_rows = page_stmt.query_map([id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;

        for page in page_rows {
            let (page_id, url, title, page_favicon) = page?;

            if let Some(ref filter) = search_filter {
                let title_lower = title.to_lowercase();
                let url_lower = url.to_lowercase();
                if !title_lower.contains(filter)
                    && !url_lower.contains(filter)
                    && !hostname.to_lowercase().contains(filter)
                {
                    continue;
                }
            }

            let browsers = get_browsers_for_page(conn, page_id).unwrap_or_default();
            let display_title = if title.is_empty() || title == "New Tab" {
                truncate_url(&url, 40)
            } else {
                truncate_str(&title, 36)
            };

            let page_node_id = format!("page-{page_id}");
            links.push(GraphLink {
                source: domain_node_id.clone(),
                target: page_node_id.clone(),
            });

            nodes.push(GraphNode {
                id: page_node_id,
                node_type: "page".to_string(),
                label: display_title,
                hostname: Some(hostname.clone()),
                url: Some(url),
                title: Some(title),
                page_count: None,
                browsers,
                favicon_url: page_favicon,
                domain_id: Some(id),
            });
        }
    }

    Ok(GraphData { nodes, links })
}

pub fn get_page_detail(conn: &Connection, page_id: i64) -> SqlResult<PageDetail> {
    let (normalized_url, original_url, title, hostname, favicon_url, first_seen, last_seen): (
        String,
        String,
        String,
        String,
        Option<String>,
        String,
        String,
    ) = conn.query_row(
        "SELECT p.normalized_url, p.original_url, p.title, d.hostname, d.favicon_url,
                p.first_seen_at, p.last_seen_at
         FROM pages p JOIN domains d ON d.id = p.domain_id
         WHERE p.id = ?1",
        [page_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        },
    )?;

    let browsers = get_browsers_for_page(conn, page_id)?;

    Ok(PageDetail {
        id: page_id,
        normalized_url,
        original_url,
        title,
        hostname,
        favicon_url,
        browsers,
        first_seen_at: first_seen,
        last_seen_at: last_seen,
    })
}

pub fn set_domain_meta(
    conn: &Connection,
    domain_id: i64,
    meta_title: &str,
    meta_description: &str,
) -> SqlResult<()> {
    conn.execute(
        "UPDATE domains SET meta_title = ?1, meta_description = ?2 WHERE id = ?3",
        params![meta_title, meta_description, domain_id],
    )?;
    Ok(())
}

pub fn get_domain_detail(conn: &Connection, domain_id: i64) -> SqlResult<DomainDetail> {
    let (hostname, page_count, is_expanded, meta_title, meta_description): (
        String,
        i64,
        i64,
        Option<String>,
        Option<String>,
    ) = conn.query_row(
        "SELECT hostname, page_count, is_expanded, meta_title, meta_description FROM domains WHERE id = ?1",
        [domain_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )?;

    let mut stmt = conn.prepare(
        "SELECT id FROM pages WHERE domain_id = ?1 AND is_archived = 0 ORDER BY last_seen_at DESC",
    )?;
    let page_ids: Vec<i64> = stmt
        .query_map([domain_id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;

    let pages: Vec<PageDetail> = page_ids
        .iter()
        .filter_map(|id| get_page_detail(conn, *id).ok())
        .collect();

    let subdomains = collect_subdomains(&hostname, &pages.iter().map(|p| p.original_url.as_str()).collect::<Vec<_>>());
    let tags = list_tags_for_domain(conn, domain_id)?;

    Ok(DomainDetail {
        id: domain_id,
        hostname,
        page_count,
        is_expanded: is_expanded == 1,
        meta_title,
        meta_description,
        subdomains,
        tags,
        pages,
    })
}

fn collect_subdomains(registrable: &str, urls: &[&str]) -> Vec<String> {
    let mut subdomains: Vec<String> = Vec::new();

    for url in urls {
        let host = Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_lowercase()));

        if let Some(host) = host {
            if host != registrable
                && crate::normalize::registrable_domain(&host) == registrable
                && !subdomains.contains(&host)
            {
                subdomains.push(host);
            }
        }
    }

    subdomains.sort();
    subdomains
}

pub fn export_json(conn: &Connection) -> SqlResult<String> {
    let mut domain_stmt = conn.prepare(
        "SELECT id, hostname, meta_title, meta_description
         FROM domains WHERE page_count > 0 ORDER BY hostname",
    )?;
    let domain_rows = domain_stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;

    let mut domains = Vec::new();
    for domain in domain_rows {
        let (id, hostname, meta_title, meta_description) = domain?;
        let mut page_stmt = conn.prepare(
            "SELECT title, original_url FROM pages
             WHERE domain_id = ?1 AND is_archived = 0
             ORDER BY last_seen_at DESC",
        )?;
        let page_rows = page_stmt.query_map([id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut pages = Vec::new();
        for page in page_rows {
            let (title, url) = page?;
            pages.push(ExportPage {
                title: if title.is_empty() || title == "New Tab" {
                    url.clone()
                } else {
                    title
                },
                url,
            });
        }

        let subdomains = collect_subdomains(
            &hostname,
            &pages.iter().map(|p| p.url.as_str()).collect::<Vec<_>>(),
        );

        let tag_names: Vec<String> = list_tags_for_domain(conn, id)?
            .into_iter()
            .map(|t| t.name)
            .collect();

        domains.push(ExportDomain {
            hostname,
            title: empty_to_none(meta_title),
            description: empty_to_none(meta_description),
            subdomains,
            tags: tag_names,
            pages,
        });
    }

    let all_tags: Vec<String> = list_tags(conn)?
        .into_iter()
        .map(|t| t.name)
        .collect();

    serde_json::to_string_pretty(&ExportTree {
        tags: all_tags,
        domains,
    })
    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
}

pub fn import_json(conn: &mut Connection, raw: &str) -> Result<(usize, usize), String> {
    let tree = parse_export_tree(raw)?;
    let now = chrono::Utc::now().to_rfc3339();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut domains_upserted = 0usize;
    let mut pages_upserted = 0usize;

    for name in &tree.tags {
        let _ = create_tag(&tx, name);
    }

    for domain in tree.domains {
        let hostname = crate::normalize::registrable_domain(&domain.hostname);
        if hostname.is_empty() {
            continue;
        }

        let domain_id = upsert_domain(&tx, &hostname, &now).map_err(|e| e.to_string())?;
        domains_upserted += 1;

        if let Some(title) = empty_to_none(domain.title.clone()) {
            tx.execute(
                "UPDATE domains SET meta_title = ?1
                 WHERE id = ?2 AND (meta_title IS NULL OR meta_title = '')",
                params![title, domain_id],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(description) = empty_to_none(domain.description.clone()) {
            tx.execute(
                "UPDATE domains SET meta_description = ?1
                 WHERE id = ?2 AND (meta_description IS NULL OR meta_description = '')",
                params![description, domain_id],
            )
            .map_err(|e| e.to_string())?;
        }

        for page in domain.pages {
            let url = page.url.trim();
            if url.is_empty() {
                continue;
            }
            let Some(normalized) = crate::normalize::normalize_url(url) else {
                continue;
            };
            if is_dismissed(&tx, &normalized).unwrap_or(false) {
                continue;
            }
            upsert_page(&tx, domain_id, &normalized, url, &page.title, &now)
                .map_err(|e| e.to_string())?;
            pages_upserted += 1;
        }

        for tag_name in domain.tags {
            if let Ok(tag) = create_tag(&tx, &tag_name) {
                let _ = set_domain_tag(&tx, domain_id, tag.id, true);
            }
        }
    }

    refresh_domain_counts(&tx).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok((domains_upserted, pages_upserted))
}

fn parse_export_tree(raw: &str) -> Result<ExportTree, String> {
    if let Ok(tree) = serde_json::from_str::<ExportTree>(raw) {
        if !tree.domains.is_empty() || raw.contains("\"domains\"") {
            return Ok(tree);
        }
    }
    serde_json::from_str::<Vec<ExportDomain>>(raw)
        .map(|domains| ExportTree {
            tags: vec![],
            domains,
        })
        .map_err(|e| format!("Invalid Webgraphy JSON: {e}"))
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

fn truncate_url(url: &str, max: usize) -> String {
    truncate_str(url, max)
}
