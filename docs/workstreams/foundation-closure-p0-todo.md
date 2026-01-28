# Foundation Closure (P0) — TODO Tracker

Status: Active (workstream tracker)

This document tracks cross-workstream TODOs for `docs/workstreams/foundation-closure-p0.md`.
It is intentionally **small** and references other detailed trackers where possible.

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `FC-P0-{area}-{nnn}`

## M0 — Layout Engine v2 (multi-viewport roots)

- [ ] FC-P0-layout-001 Update `docs/layout-engine-v2-migration-inventory.md` with a “current default path” snapshot
      (what is still on the manual path vs engine-fast-path), and link the tests that guard it.
  - Primary docs: `docs/layout-engine-refactor-roadmap.md`
  - Contract gate: `docs/adr/0116-window-scoped-layout-engine-and-viewport-roots.md`
  - Evidence anchors should include at least: `crates/fret-ui/src/tree/layout.rs` and the layout conformance tests in `crates/fret-ui/src/declarative/tests/layout.rs`.

## M1 — Overlay + input arbitration (determinism)

- [ ] FC-P0-overlay-010 Add a single “conformance suite entry list” section to `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`
      that enumerates the test files + the scripted diagnostics that must stay green.
  - Primary roadmap: `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`
  - Workstream tracker: `docs/workstreams/overlay-input-arbitration-v2-todo.md`

## M2 — View-cache + prepaint windows (fearless refactor safety)

- [ ] FC-P0-prepaint-020 Define the minimal “interaction stream v1 payload” acceptance list and mark each item as:
      `In stream` / `Not in stream (still tree-walk)` / `Out of scope`.
  - Contract gate: `docs/adr/0182-prepaint-interaction-stream-and-range-reuse.md`
  - Workstream: `docs/workstreams/gpui-parity-refactor.md`
  - Initial candidates to audit:
    - cursor requests / cursor icon routing
    - outside-press observers
    - accessibility/semantics participation (even if only as “not yet”)

- [ ] FC-P0-virt-030 Add a “VirtualList window derivation roadmap snapshot” section to the GPUI parity TODO tracker:
      current behavior, target behavior, and the first surface to migrate (code view vs list/table).
  - Workstream: `docs/workstreams/gpui-parity-refactor-todo.md`
  - Contract gate: `docs/adr/0190-prepaint-windowed-virtual-surfaces.md`

## M3 — Text system v2 (UI surfaces + quality baseline)

- [ ] FC-P0-text-040 Add an “ecosystem migration checklist” table to `docs/workstreams/text-system-v2-parley.md`
      (Markdown, CodeView, Syntax highlighting spans) with per-crate status + evidence anchors.
  - Workstream: `docs/workstreams/text-system-v2-parley.md`
  - Contract gate: `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`

## M4 — Multi-window tear-off (capability-driven)

- [ ] FC-P0-mw-050 Decide where the “hovered window / set_outer_position / z-level reliability” capability quality signals live.
  - Recommendation: amend an existing accepted contract instead of creating a new ADR:
    - `docs/adr/0054-platform-capabilities-and-portability-matrix.md` (preferred), or
    - `docs/adr/0084-multi-window-degradation-policy.md`
  - Workstream context: `docs/workstreams/docking-multiwindow-imgui-parity.md`
  - TODO tracker: `docs/workstreams/docking-multiwindow-imgui-parity-todo.md`

- [ ] FC-P0-mw-060 Add a minimal scripted regression plan for tear-off parity (even if it is “manual-run only” at first).
  - Target scenarios:
    - tear off -> hover another window -> re-dock -> close empty window
    - mouse-up outside any window completes drop
    - OS close merges content back
  - Evidence should end up in: `tools/diag-scripts/*` (when feasible) or a short manual checklist section in the workstream.
