mod queries;

pub use queries::*;

use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;

pub fn open_db(path: &Path) -> SqlResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS domains (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hostname TEXT NOT NULL UNIQUE,
            favicon_url TEXT,
            is_expanded INTEGER NOT NULL DEFAULT 0,
            page_count INTEGER NOT NULL DEFAULT 0,
            last_seen_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain_id INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
            normalized_url TEXT NOT NULL UNIQUE,
            original_url TEXT NOT NULL,
            title TEXT NOT NULL DEFAULT '',
            og_image_url TEXT,
            description TEXT,
            first_seen_at TEXT NOT NULL,
            last_seen_at TEXT NOT NULL,
            is_archived INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS sync_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            tabs_found INTEGER NOT NULL DEFAULT 0,
            browsers_scanned TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS tab_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
            browser TEXT NOT NULL,
            window_id INTEGER NOT NULL,
            tab_index INTEGER NOT NULL,
            seen_at TEXT NOT NULL,
            sync_id INTEGER NOT NULL REFERENCES sync_runs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS dismissed_urls (
            normalized_url TEXT PRIMARY KEY,
            dismissed_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_pages_domain ON pages(domain_id);
        CREATE INDEX IF NOT EXISTS idx_pages_normalized ON pages(normalized_url);
        CREATE INDEX IF NOT EXISTS idx_tab_snapshots_sync ON tab_snapshots(sync_id);
        CREATE INDEX IF NOT EXISTS idx_tab_snapshots_page ON tab_snapshots(page_id);

        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE
        );

        CREATE TABLE IF NOT EXISTS domain_tags (
            domain_id INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
            tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (domain_id, tag_id)
        );
        ",
    )?;

    let _ = conn.execute("ALTER TABLE domains ADD COLUMN meta_title TEXT", []);
    let _ = conn.execute("ALTER TABLE domains ADD COLUMN meta_description TEXT", []);

    consolidate_domains(conn)?;
    let _ = conn.execute("UPDATE domains SET is_expanded = 0", []);

    Ok(())
}

fn consolidate_domains(conn: &Connection) -> SqlResult<()> {
    let mut stmt = conn.prepare("SELECT id, hostname FROM domains")?;
    let rows: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?;

    let now = chrono::Utc::now().to_rfc3339();

    for (id, hostname) in rows {
        let registrable = crate::normalize::registrable_domain(&hostname);
        if registrable == hostname {
            continue;
        }

        let target_id = queries::upsert_domain(conn, &registrable, &now)?;
        if target_id != id {
            conn.execute(
                "UPDATE pages SET domain_id = ?1
                 WHERE domain_id = ?2
                 AND normalized_url NOT IN (
                     SELECT normalized_url FROM pages WHERE domain_id = ?1
                 )",
                params![target_id, id],
            )?;
            conn.execute("DELETE FROM pages WHERE domain_id = ?1", [id])?;
            conn.execute("DELETE FROM domains WHERE id = ?1", [id])?;
        }
    }

    queries::refresh_domain_counts(conn)?;
    Ok(())
}
