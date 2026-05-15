# Code Editor Prepaint Planner Cost V1

Date: 2026-05-15
Status: Active

## Goal

Reduce resize tail cost in the code editor prepaint replay planner while preserving the current
paint outcome:

- `rows_scene_fast_miss_no_entry` stays at `0`.
- `rows_scene_full_miss_no_entry` stays at `0`.
- the replay hit rate stays at `100%` for the resize probe surface.

This is a narrow follow-on to
`docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/CLOSEOUT_AUDIT_2026-05-15.md`.
That lane removed the visible-end full miss by prebuilding the missing edge row payload before
paint. The remaining useful work is to make prepaint planning cheaper without reopening the old
lane or widening `fret-ui` contracts.

## Current Read

The current code path now trusts cached row-scene replay context for rows that already have cached
row-scene entries. That removes an extra syntax span lookup from the prepaint planner and keeps the
planner focused on validating already-cached replay state.

## Success Criteria

- `us_row_scene_prepaint_plan` drops materially from the closed-lane baseline.
- `rows_scene_prepaint_planned == rows_scene_prepaint_used` remains true on the resize probe.
- no paint-path regression reintroduces `no_entry` or `full_miss` counters.
- the lane stays code-editor-local unless a bundle points elsewhere.

## Non-Goals

- Do not reopen the closed edge-row payload-prebuild lane.
- Do not widen `CanvasPrepaintCx` or `CanvasPainter` only for this lane.
- Do not start a renderer/display-list or `VirtualList` architecture refactor without new evidence.

## Likely Next Levers

- Keep reducing planner validation cost inside `ecosystem/fret-code-editor`.
- Only split a broader architecture lane if future bundles point to renderer encoding, layout, or
  virtual-surface ownership instead of row-scene planning.
