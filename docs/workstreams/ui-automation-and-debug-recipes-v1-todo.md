---
title: UI Automation + Debug Recipes v1 (TODO)
status: draft
date: 2026-01-30
scope: diagnostics, automation, profiling, debugging
---

# UI Automation + Debug Recipes v1 (TODO)

This file tracks milestones and concrete tasks for `docs/workstreams/ui-automation-and-debug-recipes-v1.md`.

Conventions:

- “Contract” items should land with an ADR (or an update to an existing ADR).
- “Tooling” items should land with a runnable command and a reproducible demo script.

## Milestones

### M0: Workstream scaffolding (docs + gaps audit)

- [x] Add/confirm an ADR for v1 recipes + automation surface (candidate: ADR 0196).
- [x] Audit and reconcile screenshot capture paths:
  - `FRET_DIAG_SCREENSHOT=1` (bundle `frame.bmp`) vs
  - `FRET_DIAG_SCREENSHOTS=1` (PNG + manifest + request/result protocol).
- [x] Add a “known gaps” section listing current mismatches (doc vs implementation).

### M1: One-command repro packaging (`fretboard diag repro`)

- [x] Add `fretboard diag repro <script|suite>` that:
  - runs the script/suite,
  - runs post-checks,
  - emits `repro.summary.json`,
  - packs `repro.zip` with `--include-all` defaults.
- [x] Add `--with tracy` and `--with renderdoc` flags (best-effort at first).
- [x] Add an example invocation to `docs/debugging-playbook.md`.

### M2: High-level action library (Script v2 or compiler layer)

- [ ] Add intent-level actions:
  - [ ] `ensure_visible`
  - [ ] `scroll_into_view`
  - [ ] `type_text_into`
  - [ ] `menu_select`
  - [ ] `drag_to`
  - [ ] `set_slider_value`
- [ ] Decide location:
  - [ ] v2 schema in runtime (`fret-bootstrap`) vs
  - [ ] v2 compiler in tooling (`fretboard`).
- [ ] Add at least one “slider drag” demo script that is robust to DPI and window size.

### M3: Missing repaint checks (actionable failures)

- [x] Add `semantics_fingerprint` per snapshot (core hook).
- [x] Add `--check-semantics-changed-repainted` (semantics fingerprint changes must correlate with scene changes).
- [x] Add `--dump-semantics-changed-repainted-json` (machine-readable evidence for AI/CI triage).
- [x] Add optional screenshot-backed region hashing check (`--check-pixels-changed <test_id>`).
- [x] Add a UI gallery repro script exercising the pixels-changed gate (e.g. `tools/diag-scripts/ui-gallery-code-view-scroll-refresh-pixels-changed.json`).

### M4: Performance regression gates (CI/automation ready)

- [ ] Add a threshold gate:
  - [ ] `--max-top-total-us <n>`
  - [ ] `--max-top-layout-us <n>`
  - [ ] `--max-top-solve-us <n>`
- [ ] Add a stable “perf baseline” file format for selected scripts (JSON).
- [ ] Add a nightly job candidate plan (not necessarily wired in CI yet).

### M5: GPU profiling (optional, gated)

- [ ] Decide the minimal GPU timing contract (coarse per-frame vs per-pass).
- [ ] Export GPU timing into `bundle.json` behind feature flags.

## Cross-cutting hygiene

- [ ] Ensure new exports keep `bundle.json` forward-compatible (unknown fields ignored by viewer).
- [ ] Keep `fret-ui` surface policy-free (ADR 0066).
- [ ] Prefer `test_id` authoring in demos for stable automation selectors.
