use super::{collect_chromium, is_app_running, TabCollector};
use crate::models::CollectedTab;

pub struct ChromeCollector;

impl TabCollector for ChromeCollector {
    fn browser_name(&self) -> &str {
        "Chrome"
    }

    fn collect(&self) -> Result<Vec<CollectedTab>, String> {
        if !is_app_running("Google Chrome") {
            return Ok(vec![]);
        }
        collect_chromium("Google Chrome", "Chrome")
    }
}
