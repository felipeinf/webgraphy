use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedTab {
    pub url: String,
    pub title: String,
    pub browser: String,
    pub window_id: i64,
    pub tab_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSummary {
    pub sync_id: i64,
    pub tabs_found: usize,
    pub pages_upserted: usize,
    pub browsers_scanned: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_sync_at: Option<String>,
    pub total_pages: i64,
    pub total_domains: i64,
    pub open_instances: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub hostname: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub page_count: Option<i64>,
    pub browsers: Vec<String>,
    pub favicon_url: Option<String>,
    pub domain_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDetail {
    pub id: i64,
    pub normalized_url: String,
    pub original_url: String,
    pub title: String,
    pub hostname: String,
    pub favicon_url: Option<String>,
    pub browsers: Vec<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPage {
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "original_url", alias = "normalized_url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDomain {
    pub hostname: String,
    #[serde(default, alias = "meta_title")]
    pub title: Option<String>,
    #[serde(default, alias = "meta_description")]
    pub description: Option<String>,
    #[serde(default)]
    pub subdomains: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pages: Vec<ExportPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTree {
    #[serde(default)]
    pub tags: Vec<String>,
    pub domains: Vec<ExportDomain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub domains_upserted: usize,
    pub pages_upserted: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDetail {
    pub id: i64,
    pub hostname: String,
    pub page_count: i64,
    pub is_expanded: bool,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub subdomains: Vec<String>,
    pub tags: Vec<Tag>,
    pub pages: Vec<PageDetail>,
}
