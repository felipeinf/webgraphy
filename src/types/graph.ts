export interface GraphNode {
  id: string;
  node_type: "domain" | "page";
  label: string;
  hostname?: string;
  url?: string;
  title?: string;
  page_count?: number;
  browsers: string[];
  favicon_url?: string;
  domain_id?: number;
}

export interface GraphLink {
  source: string;
  target: string;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

export interface SyncSummary {
  sync_id: number;
  tabs_found: number;
  pages_upserted: number;
  browsers_scanned: string[];
  errors: string[];
}

export interface SyncStatus {
  last_sync_at: string | null;
  total_pages: number;
  total_domains: number;
  open_instances: number;
}

export interface PageDetail {
  id: number;
  normalized_url: string;
  original_url: string;
  title: string;
  hostname: string;
  favicon_url?: string;
  browsers: string[];
  first_seen_at: string;
  last_seen_at: string;
}

export interface DomainDetail {
  id: number;
  hostname: string;
  page_count: number;
  is_expanded: boolean;
  meta_title?: string | null;
  meta_description?: string | null;
  subdomains: string[];
  tags: Tag[];
  pages: PageDetail[];
}

export interface Tag {
  id: number;
  name: string;
}
