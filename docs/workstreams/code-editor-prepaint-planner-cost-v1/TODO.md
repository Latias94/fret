# TODO

Status: Closed
Date: 2026-05-16

## Done

- [x] Create a narrow follow-on lane from `code-editor-edge-row-full-path-prefetch-v1`.
- [x] Identify `us_row_scene_prepaint_plan` as the remaining visible planner hotspot.
- [x] Replace syntax-row-cache lookup during replay planning with cached replay-context validation.
- [x] Add a focused regression test that proves the planner trusts cached replay context.
- [x] Run the focused replay-plan gate.
- [x] Run resize perf and confirm paint `no_entry` / `full_miss` counters stay at `0`.
- [x] Re-run package/check/layering gates against the final code and keep the lane evidence current.
- [x] Decide that the remaining prepaint planner cost is low enough to close this lane.
- [x] Stop this lane after fresh perf evidence moved the dominant hotspot away from code-editor
      replay planning.

## Closeout Decision

- [x] Keep `code-editor-edge-row-full-path-prefetch-v1` closed.
- [x] Keep this lane closed after the cached replay-context fast path.
- [x] Continue the performance mainline in `ui-perf-zed-smoothness-v1` P1.5 (`Editor canvas paint
      replay`) with Canvas paint/cache replay and renderer payload attribution evidence.

## Stop Conditions

- Stop this lane if the dominant hotspot moves away from code-editor replay planning.
- Start a separate architecture lane if bundles point at renderer scene encoding, global layout, or
  generic virtual-surface contracts.
