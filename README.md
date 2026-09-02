# Webgraphy

[![npm version](https://img.shields.io/npm/v/webgraphy.svg)](https://www.npmjs.com/package/webgraphy)
[![npm downloads](https://img.shields.io/npm/dm/webgraphy.svg)](https://www.npmjs.com/package/webgraphy)
[![GitHub stars](https://img.shields.io/github/stars/felipeinf/webgraphy.svg)](https://github.com/felipeinf/webgraphy/stargazers)
[![License: MIT](https://img.shields.io/github/license/felipeinf/webgraphy.svg)](LICENSE)
[![platform](https://img.shields.io/badge/platform-macOS%20arm64-black.svg)](https://github.com/felipeinf/webgraphy)

macOS app that pulls your open tabs from Safari, Chrome, and Opera into a graph grouped by domain.

![Webgraphy](assets/img-0.png)

## Install

macOS Apple Silicon. Requires [Node.js](https://nodejs.org/) 18+.

```bash
npm install -g webgraphy
webgraphy
```

Or once, without installing:

```bash
npx webgraphy
```

## What it does

- Imports open tabs from Safari, Chrome, and Opera
- Deduplicates URLs across browsers
- Groups pages by domain; click a domain to see its pages
- Syncs automatically every 50 seconds, or hit **Sync now**
- Search by domain, title, or URL
- Remove pages from the graph (they will not come back on the next sync)
- Export to JSON, Markdown, or HTML bookmarks
- Double-click a page to open it in your default browser

## Permissions

The first time you sync, macOS will ask for Automation access. Allow Webgraphy to control **Safari**, **Google Chrome**, and **Opera** in **System Settings → Privacy & Security → Automation**.

Without that, live tab import is blocked. Chrome and Opera can still fall back to session files.

Your graph is stored locally at `~/.webgraphy/webgraphy.db`.
