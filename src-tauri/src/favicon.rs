use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_FAVICON_BYTES: u64 = 512 * 1024;

pub fn cache_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("favicons")
}

fn sanitize_hostname(hostname: &str) -> Option<String> {
    let trimmed = hostname.trim().to_lowercase();
    if trimmed.is_empty() || trimmed.len() > 253 {
        return None;
    }
    let valid = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !valid {
        return None;
    }
    Some(trimmed)
}

pub fn get_favicon_data_url(app_data_dir: &Path, hostname: &str) -> Result<Option<String>, String> {
    let host = match sanitize_hostname(hostname) {
        Some(h) => h,
        None => return Ok(None),
    };

    let dir = cache_dir(app_data_dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file_path = dir.join(format!("{host}.png"));

    let bytes = if file_path.exists() {
        fs::read(&file_path).map_err(|e| e.to_string())?
    } else {
        let fetched = fetch_favicon(&host)?;
        match fetched {
            Some(data) => {
                fs::write(&file_path, &data).map_err(|e| e.to_string())?;
                data
            }
            None => return Ok(None),
        }
    };

    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(&bytes)
    )))
}

fn fetch_favicon(host: &str) -> Result<Option<Vec<u8>>, String> {
    let url = format!("https://www.google.com/s2/favicons?domain={host}&sz=64");
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .build();

    let response = match agent.get(&url).call() {
        Ok(r) => r,
        Err(ureq::Error::Status(_, _)) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };

    if !response.content_type().starts_with("image/") {
        return Ok(None);
    }

    let mut buf = Vec::new();
    response
        .into_reader()
        .take(MAX_FAVICON_BYTES)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;

    Ok(Some(buf))
}
