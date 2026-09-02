# Webgraphy

Lightweight macOS app that imports open tabs from Safari, Chrome, and Opera into a deduplicated, domain-grouped force-directed graph.

## Features

- Import open tabs from Safari, Chrome, and Opera via AppleScript
- SNSS session file fallback for Chrome/Opera when AppleScript is unavailable
- URL deduplication across browsers
- Domain nodes that expand to show individual pages
- Auto-sync every 50 seconds + manual sync
- Search by domain, title, or URL
- Remove pages from the graph (dismissed URLs won't re-import)
- Export to JSON, Markdown, or HTML bookmarks
- Double-click a page node to open in your default browser

## Requirements

- macOS
- Rust toolchain
- Node.js 18+

## macOS Permissions

Grant Automation permission in **System Settings → Privacy & Security → Automation**:

- Allow Webgraphy to control **Safari**
- Allow Webgraphy to control **Google Chrome**
- Allow Webgraphy to control **Opera**

Without these permissions, live tab import won't work. The app will attempt SNSS session file fallback for Chrome and Opera.

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Data Storage

SQLite database: `~/Library/Application Support/com.felipeinf.webgraphy/webgraphy.db`

## Stack

- Tauri 2 (Rust)
- React + TypeScript + Vite
- react-force-graph-2d
- SQLite (rusqlite)
- snss-core (Chromium session parsing)
