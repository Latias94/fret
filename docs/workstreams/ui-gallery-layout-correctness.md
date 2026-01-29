# UI Gallery Layout Correctness — Tracker

Status: Draft (workstream note; ADRs remain the source of truth)

This document tracks **layout correctness** (wrong bounds, wrong sizing, wrong clipping) issues found in
`apps/fret-ui-gallery`, with a bias toward bugs that are **visually severe** and **deterministically reproducible**.

For performance investigations, see `docs/workstreams/ui-gallery-perf-scroll-measure.md`.

## 0) Goal

- Turn “looks wrong” reports into **repeatable repro bundles** + **minimal test cases**.
- Fix root causes in the correct layer (policy vs mechanism).
- Prevent regressions with scripted repros and/or unit tests.

## 1) Triage Checklist (Layout vs Visual)

Before digging into code, classify the problem:

1. **Layout issue**: bounds are wrong (hit-testing, clipping, scroll extents, overlays anchored to the wrong rect).
2. **Visual issue**: bounds are correct, but rendering is wrong (transform stack, clip/scissor, effect bounds).

Reference workflow: `docs/debugging-playbook.md`.

## 2) Collection Workflow (Preferred)

### 2.1 Capture a deterministic diagnostics bundle (recommended)

- Use `fretboard diag run` with an existing script, or add a new `tools/diag-scripts/ui-gallery-*.json`.
- If the bug is visual, capture pixels too:
  - `FRET_DIAG_SCREENSHOT=1`: enable screenshot readback and write `frame.bmp` into the most recent bundle dir when a script requests it (via `capture_screenshot`) or when dumping bundles (writes `screenshot.request`).

### 2.2 Dump the solved layout tree (when bounds are wrong)

```powershell
$env:FRET_TAFFY_DUMP=1
$env:FRET_TAFFY_DUMP_ONCE=1
$env:FRET_TAFFY_DUMP_DIR=".fret\\taffy-dumps"

# Prefer filtering by a stable semantics label when possible:
$env:FRET_TAFFY_DUMP_ROOT_LABEL="Debug:ui-gallery:resizable-panels"
```

Notes:

- Prefer a stable `SemanticsProps.label` or `test_id`-adjacent label wrapper around the region.
- Avoid dumping in perf-critical runs (dumps will stutter).

### 2.3 Compare two bundles (regressions / “only happens with toggle X”)

Use `fretboard diag compare`:

- If you want correctness only (ignoring bounds), add: `--compare-ignore-bounds`.
- If the bug is bounds-related, keep bounds enabled and set a strict epsilon: `--compare-eps-px <px>`.

## 3) Active Issues

| ID | Severity | Page | Symptom | Repro Script | Status | Owner | Notes |
|---|---:|---|---|---|---|---|---|
| L1 | P0 | `resizable` | Resizable panel group height looks wrong (ignores the intended `h_px(320)` in the demo; can collapse to ~0 when parent height is indefinite). | `tools/diag-scripts/ui-gallery-resizable-initial-bundle.json` | Fixed (pending merge) | codex | Evidence bundles: `.fret/diag-resizable-bundle/1769674921459-ui-gallery-resizable-initial/frame.bmp` (before) vs `.fret/diag-resizable-bundle/1769675245065-ui-gallery-resizable-initial/frame.bmp` (after). Debug label: `SemanticsProps.label="Debug:ui-gallery:resizable-panels"`, `test_id="ui-gallery-resizable-panels"`. |
| L2 | P0 | `intro` | “Core / UI Kit / Shadcn” preview cards (and the note) can be laid out wider than the window (tab panel expands to max-content width). | `tools/diag-scripts/ui-gallery-intro-preview-width-bundle.json` | Fixed (pending merge) | codex | Debug anchors: `label="Debug:ui-gallery:intro:preview-grid"`, `test_id="ui-gallery-intro-preview-grid"` and `label="Debug:ui-gallery:intro:preview-note"`, `test_id="ui-gallery-intro-preview-note"`. |

### L1 Notes (Resizable Panels)

- Root cause: `fret-ui-shadcn` `ResizablePanelGroup` unconditionally overwrote the caller’s size with
  `Fill`/`Fill` (percentage-like sizing). In an auto-height parent, the Fill height can resolve to ~0.
- Fix: only default to Fill when the caller did not specify width/height; do not force
  `props.layout.size` to Fill/Fill.
- Regression: `ecosystem/fret-ui-shadcn/tests/resizable_panel_group_layout.rs`.

### L2 Notes (Intro Preview Width / Tabs TabPanel)

- Root cause: `fret-ui-shadcn` `TabsContent` / `TabPanel` was auto-sized (shrink-wrapped), so
  max-content descendants could force the tab panel wider than its parent.
- Fix: make `TabsContent` default to `w_full().min_w_0()` (and keep optional `flex_1()` when
  `content_fill_remaining` is enabled).
- Regression: keep `tools/diag-scripts/ui-gallery-intro-preview-width-bundle.json` anchored via
  `ui-gallery-intro-preview-grid` / `ui-gallery-intro-preview-note` test IDs.

## 4) Next Actions

- Pick the top P0 issue and add a dedicated `tools/diag-scripts/ui-gallery-...json` repro.
- Add `SemanticsProps.test_id` / labels to make the hot region and broken bounds discoverable in bundles and dumps.
- Convert the repro into a `crates/fret-ui/src/declarative/tests/*` test when possible.
- Keep a small layout-only suite runnable via `fretboard diag suite ui-gallery-layout --launch -- cargo run -p fret-ui-gallery --release`.
