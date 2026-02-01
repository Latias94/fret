# UI Gallery Visual Parity — TODO Tracker

Status: Active (workstream tracker; keep updated as gates land)

This document tracks executable TODOs for visual parity issues in `apps/fret-ui-gallery`, with a bias toward:

- overlay placement stability (tooltip/select/menus/popovers),
- shadcn component parity issues that are visually obvious,
- deterministic repros (scripts + bundles + minimal tests).

Tracking format:

- ID: `VP-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## P0 — Overlays

- [x] VP-ovl-010 Stabilize Select item-aligned placement under wheel scroll (prevent viewport collapse/jitter).
  - Repro: `tools/diag-scripts/ui-gallery-select-wheel-scroll.json` (use `FRET_UI_GALLERY_START_PAGE=select`)
  - Evidence: regression test + fix landed in commit `e9cc45b`.
- [ ] VP-ovl-020 Add a UI Gallery script that repeats tooltip hover enter/leave N times and asserts arrow stays attached.
  - Pre-req: use `Tooltip::arrow_test_id(...)` + `Tooltip::panel_test_id(...)` so scripts can assert geometry.
  - Suggested file: `tools/diag-scripts/ui-gallery-tooltip-repeat-hover.json`
- [ ] VP-ovl-030 Add a “repeat hover” gate for HoverCard as well (same drift class).

## P1 — Controls

- [ ] VP-ctl-010 Add Slider drag repro asserting handle stays within the fill rect during back-and-forth drag.
- [ ] VP-ctl-020 Add Toggle visual alignment repro across sizes (`sm`/`default`/`lg`) and states (checked/unchecked/disabled).
- [ ] VP-ctl-030 Add Combobox menu height/padding repro across disabled/placeholder states.

## P1 — Tabs

- [ ] VP-tabs-010 Add a targeted gate for Tabs indicator + trigger padding/height (web-vs-fret preferred).

## Tooling

- [ ] VP-tool-010 Extend diagnostics action library with a high-level `repeat_hover` recipe (optional).
  - Prefer compiling to v1 script steps for compatibility.

