---
title: Diagnostics DevTools v1 - AI Workflow via MCP
status: draft
date: 2026-02-08
scope: diagnostics, devtools, mcp, automation
---

# Diagnostics DevTools v1 - AI Workflow via MCP

This document describes an end-to-end diagnostics workflow driven via the `apps/fret-devtools-mcp`
adapter (rmcp, stdio transport).

The intent is to make the existing GUI/CLI diagnostics workflow easy to automate and easy for AI
assistants to drive **without inventing new semantics**.

## Quick start

1) Start the MCP server:

```bash
cargo run -p fret-devtools-mcp
```

2) Connect a target app session.

- If the target app is running with the WebSocket diagnostics transport enabled, it will show up as
  a session in `fret_diag_sessions_list`.
- If you need the WS hub URL + token for the target app to connect, call:
  - tool: `fret_devtools_ws_info`

3) List sessions and select one:

- tool: `fret_diag_sessions_list`
- tool: `fret_diag_sessions_select`

## End-to-end AI scenario (Pick → Edit → Run → Pack → Open)

### Step 1: Enable inspect and pick a stable selector

- tool: `fret_diag_inspect_set` with `{ "enabled": true, "consume_clicks": true }`
- tool: `fret_diag_pick` (returns selector JSON)

Tip: Prefer `test_id` selectors for stability.

### Step 2: Choose a script and fork it into the user script library

- tool: `fret_diag_scripts_list`

Recommended authoring rule:

- treat `tools/diag-scripts/` as read-only (workspace library),
- copy scripts you want to edit into `.fret/diag/scripts/`.

### Step 3: Run one or more scripts

Single file:

- tool: `fret_diag_run_script_file` with `{ "script": "tools/diag-scripts/<name>.json", "timeout_ms": 120000 }`

Batch run:

- tool: `fret_diag_run` with either:
  - `{ "scripts": ["tools/diag-scripts/a.json", ".fret/diag/scripts/b.json"] }`, or
  - `{ "glob": "ui-gallery-*.json" }`

### Step 4: Pack the latest bundle and open the offline viewer

- tool: `fret_diag_pack_last_bundle` (creates a zip on disk; returns `pack_path`)
- open `tools/fret-bundle-viewer` and load the resulting `.zip` file

## MCP resources (artifacts as resources)

The MCP server exposes key artifacts as resources. These are derived from the most recent
`bundle.dumped` event observed for a given session. If resources are missing or stale, trigger a
fresh dump with `fret_diag_bundle_dump` first.

### Resource URIs

For a session `<session_id>`:

- `fret-diag://sessions/<session_id>/bundle.json`
  - JSON text for `bundle.json` (latest observed dump)
- `fret-diag://sessions/<session_id>/bundle.zip`
  - zip blob (base64) containing `bundle.json` in the same layout as `diag pack`
- `fret-diag://sessions/<session_id>/repro.summary.json`
  - JSON text (only if present on disk in the artifacts root)

The server also accepts `selected` as an alias for the default session:

- `fret-diag://selected/bundle.json`
- `fret-diag://selected/bundle.zip`
- `fret-diag://selected/repro.summary.json`

### Notes

- `bundle.zip` is generated on read from the latest `bundle.json` (it is not necessarily the full
  `diag pack` output with `_root/*` artifacts unless you ran `fret_diag_pack_last_bundle`).
- If the transport is filesystem-based (`fret_diag_connect` with `"fs"`), the latest bundle is
  sourced from `latest.txt` under the configured `FRET_DIAG_DIR`.

