use crate::models::CollectedTab;
use snss::SessionStore;
use std::path::PathBuf;

pub fn collect_from_snss(browser: &str, sessions_dir: PathBuf) -> Vec<CollectedTab> {
    if !sessions_dir.exists() {
        return vec![];
    }

    let store = match SessionStore::open_dir(&sessions_dir) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut tabs = Vec::new();
    let mut window_id = 0i64;

    for source in store.sources() {
        for window in &source.windows {
            window_id += 1;
            let mut tab_index = 0i64;
            for tab in &window.tabs {
                tab_index += 1;
                let nav = tab.current_nav();
                if nav.url.is_empty() {
                    continue;
                }
                tabs.push(CollectedTab {
                    url: nav.url.clone(),
                    title: nav.title.clone(),
                    browser: browser.to_string(),
                    window_id,
                    tab_index,
                });
            }
        }
    }

    tabs
}

pub fn collect_snss_fallback() -> (Vec<CollectedTab>, Vec<String>) {
    let home = dirs_home();
    let mut tabs = Vec::new();
    let mut errors = Vec::new();

    let chrome_sessions = home.join("Library/Application Support/Google/Chrome/Default/Sessions");
    let opera_sessions =
        home.join("Library/Application Support/com.operasoftware.Opera/Default/Sessions");

    let chrome_tabs = collect_from_snss("Chrome", chrome_sessions);
    if chrome_tabs.is_empty() {
        errors.push("Chrome SNSS: no sessions found or parse failed".to_string());
    } else {
        tabs.extend(chrome_tabs);
    }

    let opera_tabs = collect_from_snss("Opera", opera_sessions);
    if opera_tabs.is_empty() {
        errors.push("Opera SNSS: no sessions found or parse failed".to_string());
    } else {
        tabs.extend(opera_tabs);
    }

    (tabs, errors)
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}
