use super::{collect_safari, is_app_running, TabCollector};
use crate::models::CollectedTab;

pub struct SafariCollector;

impl TabCollector for SafariCollector {
    fn browser_name(&self) -> &str {
        "Safari"
    }

    fn collect(&self) -> Result<Vec<CollectedTab>, String> {
        if !is_app_running("Safari") {
            return Ok(vec![]);
        }
        collect_safari()
    }
}
