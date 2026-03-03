---
title: Diag Compatibility Matrix (Scripts, Bundles, Transports, Capabilities)
status: draft
date: 2026-02-28
scope: diagnostics, compatibility, transports, artifacts
---

# Compatibility matrix (living)

This page is a **contract inventory** for compatibility paths in the diagnostics stack.
It exists to make removals safe: when we delete a compat path, we update this matrix and the migration checklist first.

## Terms

- **Tooling**: `fretboard diag ...` (crate: `crates/fret-diag`, app: `apps/fretboard`).
- **Runtime**: in-app diagnostics service (typically `ecosystem/fret-bootstrap/src/ui_diagnostics/*`).
- **FS transport**: filesystem triggers under `.fret/diag/` (native-first).
- **WS transport**: DevTools WebSocket transport (web/wasm and remote scenarios).

## Script schema compatibility

| Item | Tooling support | Runtime support | Notes / exit plan |
| --- | --- | --- | --- |
| Script schema v2 (`UiActionScriptV2`) | Yes (default) | Yes (preferred) | This is the mainline. New scripts should be v2. |
| Script schema v1 (`UiActionScriptV1`) | Yes (manual-only; tooling can upgrade) | Yes (guarded by `allow_script_schema_v1`) | Tool-launched runs (`--launch`/`--reuse-launch`) reject v1. Exit plan: migrate in-repo scripts to v2 (legacy `script_redirect` stubs may remain v1). |
| `script_redirect` stubs (`kind: script_redirect`) | Yes (tooling resolves; loop detection) | No (should never reach runtime) | Used for fearless path moves (tooling-only). Suites are expressed via suite manifests. |

## Artifact / bundle schema compatibility

| Item | Tooling support | Runtime writes | Notes / exit plan |
| --- | --- | --- | --- |
| Per-run manifest (`manifest.json`, schema v2) | Yes (preferred) | Yes | Transport-neutral run layout; enables “small-by-default” tooling. |
| Bundle schema v2 view (`bundle.schema2.json`) | Yes | Optional (config-driven) | Compatibility view; should become more default as legacy writers are turned off. |
| Bundle schema v1 view (`bundle.json`) | Yes (compat) | Legacy / transitional | Exit plan: stop writing by default (P3), keep opt-in for downstream users until migrations complete. |
| Sidecars (`bundle.meta.json`, indexes, script.result.json) | Yes (preferred) | Yes | Tooling should triage via meta/index/slice, not by grepping `bundle.json`. |

## Transport compatibility

| Item | Tooling support | Runtime support | Notes / exit plan |
| --- | --- | --- | --- |
| FS transport (triggers) | Yes | Yes | Default for native dev loops. |
| WS transport (DevTools) | Yes | Yes | Required for web/wasm exports. |
| Dump request metadata parity (label/max snapshots/request id) | Yes | Yes | FS uses `dump.request.json` + trigger; WS uses request-id correlation. |

## Capabilities (selected)

| Capability | Meaning | Where enforced |
| --- | --- | --- |
| `diag.script_v2` | runtime supports script schema v2 | runtime + tooling validation |
| `diag.screenshot_png` | runtime can write PNG screenshots for `capture_screenshot` | tooling infers from steps; runtime requires screenshots enabled |
| `diag.pointer_kind_touch`, `diag.pointer_kind_pen` | pointer-kind injection support | tooling infers from steps; runtime synthesizes pointer events |
| `diag.multi_window` | multi-window targeting supported | tooling gates when `window` targets require it |
