# Webgraphy

[![npm version](https://img.shields.io/npm/v/webgraphy.svg)](https://www.npmjs.com/package/webgraphy)
[![npm downloads](https://img.shields.io/npm/dm/webgraphy.svg)](https://www.npmjs.com/package/webgraphy)
[![GitHub stars](https://img.shields.io/github/stars/felipeinf/webgraphy.svg)](https://github.com/felipeinf/webgraphy/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/felipeinf/webgraphy.svg)](https://github.com/felipeinf/webgraphy/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/felipeinf/webgraphy.svg)](https://github.com/felipeinf/webgraphy)
[![License: MIT](https://img.shields.io/github/license/felipeinf/webgraphy.svg)](LICENSE)
[![platform](https://img.shields.io/badge/platform-macOS%20arm64-black.svg)](https://github.com/felipeinf/webgraphy)

Lightweight macOS app that imports open tabs from Safari, Chrome, and Opera into a deduplicated, domain-grouped force-directed graph.

![Webgraphy](assets/img-0.png)

## Install

macOS Apple Silicon (arm64):

```bash
npm install -g webgraphy
webgraphy
```

Or run without installing:

```bash
npx webgraphy
```

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

SQLite database: `~/.webgraphy/webgraphy.db`

## Stack

- Tauri 2 (Rust)
- React + TypeScript + Vite
- react-force-graph-2d
- SQLite (rusqlite)
- snss-core (Chromium session parsing)
