# Code Editor Prepaint Planner Cost V1

Date: 2026-05-15
Status: Closed on 2026-05-16

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

## Closeout Read

This lane is closed. The cached replay-context fast path reduced
`us_row_scene_prepaint_plan` from `91us` to `67us` p95 on the carried resize evidence while keeping
the paint miss invariants at `0`.

Fresh post-merge evidence then moved the dominant editor perf question away from prepaint planning:
the current typical autoscroll sample is paint/widget dominated, with hot row-scene replay already
at `100%`. Continue editor performance work from
`docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-todo.md` P1.5
(`Editor canvas paint replay`) rather than reopening this prepaint-planner folder.

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

- Keep this lane closed unless a future bundle again proves prepaint replay planning is the
  dominant bottleneck.
- Continue the main perf line through Editor Canvas paint/cache replay evidence, then split a
  renderer/display-list or virtual-surface owner lane only if that evidence points outside
  `fret-code-editor`.
