# TODO

Status: Active after M1
Date: 2026-05-15

## Done

- [x] Create the narrow follow-on lane after `code-editor-row-content-snapshot-cache-v1`.
- [x] Add plain cached row support to prepaint row scene replay planning.
- [x] Add a focused regression test for plain cached row replay planning during resize.
- [x] Run focused/package code-editor tests, check, format check, diff check, and resize perf.
- [x] Record M1 evidence and residual risk.

## Next Executable Slices

- [ ] M2: add a small miss taxonomy for resize edge rows.
  - Record how many paint frames miss because the row has no row scene cache entry, syntax replay
    validation fails, rich/geometry keys drift, or candidate planning skips the row.
  - Keep counters behind existing code-editor paint perf diagnostics.
- [ ] M2: make replay-plan candidate selection edge-aware.
  - Prefer newly exposed viewport edge rows and cached rows that are likely to be consumed in the
    same frame.
  - Avoid increasing `us_row_scene_prepaint_plan` while chasing a small row-content win.
- [ ] M3: only if M2 evidence still points there, design a true edge-row payload prebuild.
  - Decide whether the payload can be built from existing row rich/geom caches without new
    `CanvasPainter` or `Scene` API surface.
  - If new API is required, split it into a separate contract-first lane.

## Stop Conditions

- Stop this lane if worst-frame attribution moves away from code-editor row content/rich/geom/scene
  work.
- Start a separate architecture lane if the evidence points at `fret-ui` layout, view-cache,
  `Scroll`, `VirtualList`, or renderer contracts.
- Close this lane if another slice cannot reduce code-editor paint p95 without increasing prepaint
  or layout p95 enough to erase the win.
