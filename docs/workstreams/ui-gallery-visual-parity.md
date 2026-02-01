---
title: UI Gallery Visual Parity (Shadcn + Overlays)
status: active
date: 2026-02-01
scope: ui-gallery, shadcn, overlays, visual-parity
---

# UI Gallery Visual Parity (Shadcn + Overlays)

This workstream tracks **visual parity** issues in `apps/fret-ui-gallery` that are not strictly “layout correctness”
(bounds are often *technically* valid, but the visuals or placement feel wrong).

For deterministic **layout correctness** gates, see `docs/workstreams/ui-gallery-layout-correctness.md`.
For performance investigations, see `docs/workstreams/ui-gallery-perf-scroll-measure.md`.

## Goals

- Turn “looks wrong” reports into **deterministic repro scripts** (`tools/diag-scripts/ui-gallery-*.json`).
- Prefer **geometry/semantics assertions** over screenshot goldens for day-to-day iteration.
- When the target is shadcn parity, prefer **web-vs-fret goldens** (or minimal overlay placement tests) as the long-term gate.

## Non-goals

- Expanding `fret-ui` with policy (keep mechanism vs policy layering intact; ADR 0066).
- Making UI Gallery “perfect” before contracts are stable; focus on the highest signal defects first.

## Workflow (Preferred)

1. Add/extend a `tools/diag-scripts/ui-gallery-*.json` repro.
2. If the issue is hard to target, add minimal `test_id` anchors at the component layer (shadcn/ecosystem first).
3. Capture a bundle + screenshot evidence via `fretboard diag run`.
4. Fix in the correct layer (mechanism vs policy).
5. Add a regression gate (script + assertion, or a unit/integration test).

## Current Issues (Shortlist)

| ID | Severity | Surface | Symptom | Repro | Status |
|---|---:|---|---|---|---|
| VP-001 | P0 | Select | Wheel scroll after opening causes menu rect/viewport to jitter or collapse. | `tools/diag-scripts/ui-gallery-select-wheel-scroll.json` (set `FRET_UI_GALLERY_START_PAGE=select`) | Fixed (commit `e9cc45b`) |
| VP-002 | P0 | Tooltip | After repeated hover cycles, arrow/diamond visually separates from the tooltip panel. | `tools/diag-scripts/ui-gallery-tooltip-repeat-hover.json` (set `FRET_UI_GALLERY_START_PAGE=overlay`) | Investigating (repro script landed in `7ac4088`) |
| VP-003 | P1 | Slider | Dragging can visually desync handle vs fill. | `tools/diag-scripts/ui-gallery-slider-range-drag-stability.json` (set `FRET_UI_GALLERY_START_PAGE=slider`) | Gated (script + anchors in `d56128f` / `9062e00`) |
| VP-004 | P2 | Toggle | Knob appears slightly misaligned (right/down) relative to track. | Needs a new script | Open |
| VP-005 | P1 | Combobox | Dropdown height/padding differs; disabled text baseline too tight. | Needs a new script | Open |
| VP-006 | P1 | Tabs | Visual styling differs from upstream (indicator/spacing). | Needs a new script | Open |
| VP-007 | P1 | UI Gallery perf | Clicking card feels delayed (~0.5s). | Use `fretboard diag perf` + targeted page start | Tracked in perf workstream |

## Recent Instrumentation (to enable repros)

- Tooltip now supports `arrow_test_id(...)` and `panel_test_id(...)` for diagnostics (commit `c3f43b1`).
  - Code: `ecosystem/fret-ui-shadcn/src/tooltip.rs`.
- Select trigger in UI Gallery exposes a stable automation id: `ui-gallery-select-trigger` (commit `e9cc45b`).
  - Code: `apps/fret-ui-gallery/src/ui.rs`.
- Slider now exposes `test_id`-derived internals: `{id}-track`, `{id}-range`, `{id}-thumb-{i}` (commit `d56128f`).
  - Code: `ecosystem/fret-ui-shadcn/src/slider.rs`.
- Diagnostics gained axis-only overlap predicates (`bounds_overlapping_x`/`bounds_overlapping_y`) for cases where vertical offset is expected (commit `0fdea34`).
  - Code: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.
