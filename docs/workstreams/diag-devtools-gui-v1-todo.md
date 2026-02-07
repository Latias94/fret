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

- [ ] Add `apps/fret-devtools` (native target first, but structured for web runner).
- [ ] Implement the minimal 4-panel UX:
  - [ ] Inspect (inspect toggle + pick + show selector JSON),
  - [ ] Scripts (browse `tools/diag-scripts` + open editor),
  - [ ] Run (run a script; show progress + failures),
  - [ ] Artifacts (latest bundle + pack + open viewer).
- [ ] Add a “watch” loop for `FRET_DIAG_DIR` updates (native transport):
  - [ ] auto-refresh latest bundle,
  - [ ] auto-refresh `pick.result.json`, `script.result.json`, screenshot results.

### M3: Script Studio (authoring UX)

- [ ] Schema-aware script editor:
  - [ ] validate schema version,
  - [ ] step palette (v1 steps + v2 intent steps),
  - [ ] structured editor for selector/predicate.
- [ ] Pick-to-fill UX:
  - [ ] select a JSON pointer (e.g. `/steps/3/target`) and apply pick result into the editor (equivalent of `diag pick-apply`).
- [ ] Script library ergonomics:
  - [ ] fork/copy a script into `.fret/diag/scripts/` (avoid editing workspace scripts by default),
  - [ ] “Run this script” button produces a shareable zip by default.

### M4: WebSocket transport (enables web runner)

- [ ] Implement WS server (DevTools side):
  - [ ] binds to `127.0.0.1`,
  - [ ] requires a capability token,
  - [ ] supports multiple clients (session ids).
- [ ] Implement WS client bridge in diagnostics service:
  - [ ] add `FRET_DEVTOOLS_WS=ws://127.0.0.1:<port>` (name TBD) to enable it,
  - [ ] wasm32 client via `web_sys::WebSocket`,
  - [ ] native client via a non-blocking reader thread + queue (avoid blocking the frame loop).
  - [ ] web runner config: support query string and/or `window.__FRET_DEVTOOLS_WS` globals (name TBD).
- [ ] Map protocol commands to existing in-app operations:
  - [ ] inspect config updates,
  - [ ] pick arm + pick result,
  - [ ] script push + script progress + script result,
  - [ ] bundle dump + screenshot request.

### M5: Artifacts for web runner

- [ ] Define an artifact store abstraction for diagnostics outputs:
  - [ ] native: filesystem (existing),
  - [ ] web: in-memory store + “download zip” export.
- [ ] Ensure the offline bundle viewer can open zips produced by web runs (same structure as `diag pack`).

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

- [ ] Add `apps/fret-devtools-mcp` using `rmcp` (stdio transport first).
- [ ] Expose a minimal tool set (names TBD):
  - [ ] `fret_diag_connect` (connect/select session; choose transport: filesystem vs WS),
  - [ ] `fret_diag_inspect_set` (on/off/toggle + consume_clicks),
  - [ ] `fret_diag_pick` (arm + wait + return selector JSON),
  - [ ] `fret_diag_run` (run script/suite/repro/perf with options; return summary + evidence paths),
  - [ ] `fret_diag_pack` / `fret_diag_latest` / `fret_diag_compare`.
- [ ] (Optional) Expose key artifacts as resources:
  - [ ] latest `bundle.json`,
  - [ ] `repro.summary.json`,
  - [ ] packed zip bytes (or a download handle).
- [ ] Add an end-to-end AI scenario doc:
  - [ ] “Pick selector → patch script → run → pack → open viewer” driven via MCP tools.

## Cross-cutting hygiene

- [ ] Keep `bundle.json` forward-compatible (unknown fields ignored by viewer).
- [ ] Keep `fret-ui` policy-free; DevTools policy stays in `ecosystem/*` and apps/tooling.
- [ ] Prefer authoring `test_id` in recipes to make scripts stable.
