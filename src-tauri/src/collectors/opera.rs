use super::{collect_chromium, is_app_running, TabCollector};
use crate::models::CollectedTab;

pub struct OperaCollector;

const OPERA_APPS: &[&str] = &["Opera", "Opera GX"];

impl TabCollector for OperaCollector {
    fn browser_name(&self) -> &str {
        "Opera"
    }

    fn collect(&self) -> Result<Vec<CollectedTab>, String> {
        let mut tabs = Vec::new();
        let mut last_error: Option<String> = None;

        for app in OPERA_APPS {
            if !is_app_running(app) {
                continue;
            }
            match collect_chromium(app, "Opera") {
                Ok(found) => tabs.extend(found),
                Err(e) => last_error = Some(e),
            }
        }

        if tabs.is_empty() {
            if let Some(e) = last_error {
                return Err(e);
            }
        }
        Ok(tabs)
    }
}
