---
title: Diagnostics DevTools GUI v1 (TODO)
status: draft
date: 2026-02-07
scope: diagnostics, automation, devtools, web-runner
---

# Diagnostics DevTools GUI v1 (TODO)

This file tracks milestones and executable tasks for `docs/workstreams/diag-devtools-gui-v1.md`.

Conventions:

- Prefer “extract and reuse” over re-implementing logic inside the GUI.
- Every milestone should end with a runnable demo path (native and web when applicable).

## Milestones

### M0: Scaffolding + decisions (docs + contracts)

- [x] Add this workstream doc + TODO tracker.
- [x] Decide WS topology for web runner support:
  - [x] DevTools hosts local WS server (recommended).
  - [x] Session token defaults (origin allowlist is implementation-time polish).
- [x] Decide the initial protocol framing:
  - [x] `{"schema_version":1,"type":"...","request_id":...,"payload":...}` (or similar).
  - [x] Correlation rules for request/response vs push events.
- [x] Decide protocol naming and limits:
  - [x] env var + query string keys for web runner.
  - [x] message type naming convention (`inspect.set` vs `inspect_set`).
  - [x] max message size + hover event backpressure rules.
- [x] Decide the default tree shown in the left panel:
  - [x] semantics tree (recommended default),
  - [ ] layout tree (debugging layout engine),
  - [ ] element tree (authoring identity / caching boundaries).

### M1: Extract reusable tooling into crates

- [x] Create `crates/fret-diag-protocol` (wasm32-compatible, no std::fs required).
  - [x] Move script/selector/predicate types into it (schema v1/v2).
  - [x] Add serde roundtrip tests for representative scripts from `tools/diag-scripts/`.
- [x] Create `crates/fret-diag` and move core logic from `apps/fretboard` diag CLI into it:
  - [x] pack/share helpers (zip + `_root/` artifacts),
  - [x] bundle stats + gates + compare,
  - [x] file-trigger helpers (touch/write/wait) for the existing transport.
- [x] Make `apps/fretboard` depend on `crates/fret-diag` and keep CLI behavior identical.

### M2: New GUI app skeleton (`apps/fret-devtools`)

- [x] Add `apps/fret-devtools` (native target first, but structured for web runner).
- [x] Implement the minimal 4-panel UX:
  - [x] Inspect (inspect toggle + pick + show selector JSON),
  - [x] Run (run a script; show progress + failures),
  - [x] Artifacts (latest bundle dump payload),
  - [x] Scripts (browse `tools/diag-scripts` + open editor).
  - [x] Semantics tree (virtualized via `VirtualList`; keep selection visible; selected-node inspector).
  - [x] WS message tail (basic event log).
- [x] Add a “watch” loop for `FRET_DIAG_DIR` updates (native transport):
  - [x] auto-refresh latest bundle,
  - [x] auto-refresh `pick.result.json`, `script.result.json`, screenshot results.

### M3: Script Studio (authoring UX)

- [x] Schema-aware script editor:
  - [x] validate schema version (v1/v2) on push/run,
  - [x] step palette (v1 steps + v2 intent steps),
  - [x] structured editor for selector/predicate.
- [x] Pick-to-fill UX:
  - [x] select a JSON pointer (e.g. `/steps/3/target`) and apply pick result into the editor (equivalent of `diag pick-apply`).
  - [x] discover pointer candidates from the current script (faster pointer targeting).
- [x] Script library ergonomics:
  - [x] fork/copy a script into `.fret/diag/scripts/` (avoid editing workspace scripts by default),
  - [x] “Run this script” can produce a shareable zip (Run & Pack / Pack last bundle).

### M4: WebSocket transport (enables web runner)

- [x] Implement WS server (DevTools side):
  - [x] binds to `127.0.0.1`,
  - [x] requires a capability token,
  - [x] supports multiple clients (session ids).
- [x] Implement WS client bridge in diagnostics service:
  - [x] add `FRET_DEVTOOLS_WS=ws://127.0.0.1:<port>` (name TBD) to enable it,
  - [x] wasm32 client via `web_sys::WebSocket`,
  - [x] native client via a non-blocking reader thread + queue (avoid blocking the frame loop).
  - [x] web runner config: support query string and/or `window.__FRET_DEVTOOLS_WS` globals (name TBD).
- [x] Map protocol commands to existing in-app operations:
  - [x] inspect config updates,
  - [x] pick arm + pick result,
  - [x] script push + script progress + script result,
  - [x] bundle dump,
  - [x] screenshot request.
  - [x] semantics node details on-demand (`semantics.node.get` / `semantics.node.get_ack`).

### M5: Artifacts for web runner

- [x] Define an artifact store abstraction for diagnostics outputs:
  - [x] native: filesystem (existing),
  - [x] web: in-memory export helpers (zip bytes + materialize to exports dir).
- [x] Allow DevTools to pack web runner dumps by materializing `bundle.dumped.bundle` into `.fret/diag/exports/`.
- [x] Ensure the offline bundle viewer can open zips produced by web runs (same structure as `diag pack`).

### M6: Quality gates + “real-time inspect” polish

- [ ] First-class UI for gates:
  - [ ] stale paint/scene,
  - [ ] pixels changed,
  - [ ] perf thresholds,
  - [ ] resource footprint thresholds.
- [ ] Live inspect payloads (keep minimal):
  - [ ] hovered node summary + bounds,
  - [ ] focus node summary,
  - [ ] overlay barrier root id + blocking roots summary.
- [ ] Add at least one “dogfood” demo workflow:
  - [ ] open UI gallery, pick a button, generate a script, run it, pack zip, open viewer.
 - [ ] Validate tree scalability:
   - [ ] virtualized rendering for 50k+ semantics nodes,
   - [ ] low-traffic live updates (operations/polling) under scroll.

### M7: MCP server adapter (AI-friendly tooling API)

- [x] Add `apps/fret-devtools-mcp` using `rmcp` (stdio transport first).
- [x] Expose a minimal tool set (names TBD):
  - [x] `fret_diag_sessions_list` (list sessions),
  - [x] `fret_diag_sessions_select` (select a session),
  - [x] `fret_diag_connect` (choose transport: filesystem vs WS; optional),
  - [x] `fret_diag_inspect_set` (on/off/toggle + consume_clicks),
  - [x] `fret_diag_pick` (arm + wait + return selector JSON),
  - [x] `fret_diag_scripts_list` (list `tools/diag-scripts` and `.fret/diag/scripts`),
  - [x] `fret_diag_run_script_file` (run a script by file name or relative path),
  - [x] `fret_diag_run` (run multiple scripts with list/glob; returns structured summary),
  - [x] `fret_diag_run_script_json` (minimal: run a v1/v2 script JSON and wait for pass/fail),
  - [x] `fret_diag_pack_last_bundle` (dump + pack into zip),
  - [x] `fret_diag_pack_last_bundle_zip_bytes` (dump + pack bundle.json zip bytes as base64),
  - [x] `fret_diag_bundle_dump_latest` (best-effort latest bundle.dumped payload),
  - [x] `fret_diag_compare`.
- [x] (Optional) Expose key artifacts as resources:
  - [x] latest `bundle.json`,
  - [x] `repro.summary.json` (when present on disk),
  - [x] `bundle.zip` (generated on read; same layout as `diag pack`).
- [x] Support resource subscriptions + notifications for artifact updates.
- [ ] Add an end-to-end AI scenario doc:
  - [x] “Pick selector → patch script → run → pack → open viewer” driven via MCP tools.

## Cross-cutting hygiene

- [ ] Keep `bundle.json` forward-compatible (unknown fields ignored by viewer).
- [ ] Keep `fret-ui` policy-free; DevTools policy stays in `ecosystem/*` and apps/tooling.
- [ ] Prefer authoring `test_id` in recipes to make scripts stable.
