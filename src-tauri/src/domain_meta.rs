use std::time::Duration;

const MAX_HTML_BYTES: usize = 256 * 1024;

pub fn fetch_domain_meta(hostname: &str) -> Option<(String, String)> {
    let url = format!("https://{}/", hostname.trim());
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(4))
        .redirects(2)
        .build();

    let response = match agent.get(&url).call() {
        Ok(r) => r,
        Err(ureq::Error::Status(_, _)) => return None,
        Err(_) => return None,
    };

    let mut html = String::new();
    let mut reader = response.into_reader();
    let mut buf = [0u8; 8192];
    let mut total = 0usize;

    loop {
        let read = match std::io::Read::read(&mut reader, &mut buf) {
            Ok(n) => n,
            Err(_) => break,
        };
        if read == 0 {
            break;
        }
        total += read;
        if total > MAX_HTML_BYTES {
            break;
        }
        html.push_str(&String::from_utf8_lossy(&buf[..read]));
    }

    if html.is_empty() {
        return None;
    }

    let title = extract_meta_content(&html, "og:title")
        .or_else(|| extract_meta_content(&html, "twitter:title"))
        .or_else(|| extract_title_tag(&html))
        .unwrap_or_default();

    let description = extract_meta_content(&html, "og:description")
        .or_else(|| extract_meta_content(&html, "description"))
        .or_else(|| extract_meta_content(&html, "twitter:description"))
        .unwrap_or_default();

    if title.is_empty() && description.is_empty() {
        return None;
    }

    Some((title, description))
}

fn extract_meta_content(html: &str, key: &str) -> Option<String> {
    let patterns = [
        format!("property=\"{key}\""),
        format!("name=\"{key}\""),
        format!("property='{key}'"),
        format!("name='{key}'"),
    ];

    for pattern in patterns {
        if let Some(value) = extract_content_after_pattern(html, &pattern) {
            let trimmed = decode_entities(value.trim());
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

fn extract_content_after_pattern(html: &str, pattern: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    let idx = lower.find(&pattern_lower)?;
    let slice = &html[idx..];
    let content_idx = slice.to_lowercase().find("content=")?;
    let after = &slice[content_idx + 8..];
    let first = after.chars().next()?;
    if first == '"' || first == '\'' {
        let end = after[1..].find(first)?;
        return Some(after[1..1 + end].to_string());
    }
    None
}

fn extract_title_tag(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title>")?;
    let end = lower[start + 7..].find("</title>")?;
    let title = html[start + 7..start + 7 + end].trim();
    if title.is_empty() {
        None
    } else {
        Some(decode_entities(title))
    }
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
