use crate::models::CollectedTab;
use std::process::Command;

pub trait TabCollector {
    fn browser_name(&self) -> &str;
    fn collect(&self) -> Result<Vec<CollectedTab>, String>;
}

pub mod chrome;
pub mod opera;
pub mod safari;

const CHROMIUM_JXA: &str = r#"
function run(argv) {
  const app = Application(argv[0]);
  const lines = [];
  const windows = app.windows();
  for (let wi = 0; wi < windows.length; wi++) {
    const w = windows[wi];
    let winId = wi + 1;
    try { winId = w.id(); } catch (e) {}
    let tabs = [];
    try { tabs = w.tabs(); } catch (e) { continue; }
    for (let ti = 0; ti < tabs.length; ti++) {
      const t = tabs[ti];
      let url = "";
      let title = "";
      try { url = t.url() || ""; } catch (e) {}
      try { title = t.title() || ""; } catch (e) {}
      if (!url) continue;
      lines.push([url, title, String(winId), String(ti + 1)]
        .map(s => String(s).replace(/[\t\n\r]/g, " ")).join("\t"));
    }
  }
  return lines.join("\n");
}
"#;

const SAFARI_JXA: &str = r#"
function run() {
  const app = Application("Safari");
  const lines = [];
  const windows = app.windows();
  for (let wi = 0; wi < windows.length; wi++) {
    const w = windows[wi];
    let winId = wi + 1;
    try { winId = w.id(); } catch (e) {}
    let tabs = [];
    try { tabs = w.tabs(); } catch (e) { continue; }
    for (let ti = 0; ti < tabs.length; ti++) {
      const t = tabs[ti];
      let url = "";
      let title = "";
      try { url = t.url() || ""; } catch (e) {}
      try { title = t.name() || ""; } catch (e) {}
      if (!url) continue;
      lines.push([url, title, String(winId), String(ti + 1)]
        .map(s => String(s).replace(/[\t\n\r]/g, " ")).join("\t"));
    }
  }
  return lines.join("\n");
}
"#;

pub fn run_jxa(script: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-l")
        .arg("JavaScript")
        .arg("-e")
        .arg(script)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run osascript: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("JXA error: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn collect_chromium(app_name: &str, browser_label: &str) -> Result<Vec<CollectedTab>, String> {
    let output = run_jxa(CHROMIUM_JXA, &[app_name])?;
    Ok(parse_tab_lines(&output, browser_label))
}

pub fn collect_safari() -> Result<Vec<CollectedTab>, String> {
    let output = run_jxa(SAFARI_JXA, &[])?;
    Ok(parse_tab_lines(&output, "Safari"))
}

pub fn is_app_running(app_name: &str) -> bool {
    let output = Command::new("pgrep")
        .arg("-x")
        .arg(app_name)
        .output();
    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn parse_tab_lines(output: &str, browser: &str) -> Vec<CollectedTab> {
    let mut tabs = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 {
            continue;
        }
        let url = parts[0].trim().to_string();
        let title = parts[1].trim().to_string();
        let window_id = parts[2].trim().parse().unwrap_or(0);
        let tab_index = parts[3].trim().parse().unwrap_or(0);

        if url.is_empty() {
            continue;
        }

        tabs.push(CollectedTab {
            url,
            title,
            browser: browser.to_string(),
            window_id,
            tab_index,
        });
    }
    tabs
}

pub fn collect_all() -> (Vec<CollectedTab>, Vec<String>) {
    let collectors: Vec<Box<dyn TabCollector>> = vec![
        Box::new(safari::SafariCollector),
        Box::new(chrome::ChromeCollector),
        Box::new(opera::OperaCollector),
    ];

    let mut all_tabs = Vec::new();
    let mut errors = Vec::new();

    for collector in collectors {
        match collector.collect() {
            Ok(tabs) => all_tabs.extend(tabs),
            Err(e) => errors.push(format!("{}: {e}", collector.browser_name())),
        }
    }

    (all_tabs, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tab_separated_lines() {
        let input = "https://a.com\tA\t7\t1\nhttps://b.com\tB\t7\t2\n\nbad line\n";
        let tabs = parse_tab_lines(input, "Opera");
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].url, "https://a.com");
        assert_eq!(tabs[0].window_id, 7);
        assert_eq!(tabs[1].tab_index, 2);
        assert_eq!(tabs[1].browser, "Opera");
    }
}
