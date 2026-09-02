use url::Url;

const TRACKING_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_id",
    "fbclid",
    "gclid",
    "gclsrc",
    "msclkid",
    "mc_cid",
    "mc_eid",
    "_ga",
    "ref",
];

pub fn normalize_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "about:blank" {
        return None;
    }

    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else if trimmed.starts_with("chrome://")
        || trimmed.starts_with("safari-extension://")
        || trimmed.starts_with("opera://")
        || trimmed.starts_with("about:")
    {
        return None;
    } else {
        format!("https://{trimmed}")
    };

    let mut parsed = Url::parse(&with_scheme).ok()?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return None;
    }

    if let Some(host) = parsed.host_str() {
        let lower = host.to_lowercase();
        let stripped = lower.strip_prefix("www.").unwrap_or(&lower);
        let _ = parsed.set_host(Some(stripped));
    }

    parsed.set_fragment(None);

    let mut pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|(key, _)| !is_tracking_param(&key))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    parsed.set_query(None);
    if !pairs.is_empty() {
        let query: String = pairs
            .iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    urlencoding_encode(k)
                } else {
                    format!("{}={}", urlencoding_encode(k), urlencoding_encode(v))
                }
            })
            .collect::<Vec<_>>()
            .join("&");
        parsed.set_query(Some(&query));
    }

    let mut normalized = parsed.to_string();
    if normalized.ends_with('/') && parsed.path() != "/" {
        normalized.pop();
    }

    Some(normalized)
}

pub fn extract_hostname(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
}

fn is_tracking_param(key: &str) -> bool {
    let lower = key.to_lowercase();
    TRACKING_PARAMS
        .iter()
        .any(|p| lower == *p || lower.starts_with("utm_"))
}

fn urlencoding_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_www_and_tracking() {
        let url = "https://www.youtube.com/watch?v=abc&utm_source=twitter";
        let norm = normalize_url(url).unwrap();
        assert_eq!(norm, "https://youtube.com/watch?v=abc");
    }

    #[test]
    fn dedupes_trailing_slash() {
        let a = normalize_url("https://github.com/user/").unwrap();
        let b = normalize_url("https://github.com/user").unwrap();
        assert_eq!(a, b);
    }
}
