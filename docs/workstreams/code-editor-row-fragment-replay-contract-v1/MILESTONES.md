# Code Editor Row Fragment Replay Contract v1 Milestones

Status: Active
Date: 2026-05-16

## M0 - Lane Scaffold And Baseline Evidence

Status: Complete

Exit criteria:

- Workstream docs exist.
- Current local attribution and rejected micro-cleanup evidence are recorded.
- Gates are explicit.

## M1 - Target Shape Decision

Status: Complete

Decide between:

- a contiguous run descriptor over existing per-row cached scene fragments,
- or a precomposed visible-window scene fragment.

The decision must name:

- fallback behavior,
- overlay/preedit handling,
- hosted-resource touch semantics,
- boundary diagnostics fields.

Decision: use a retained per-row fragment reference first. Each `RowSceneReplayPlanEntry` carries
the row, local bounds, and an `Arc<RowSceneRetainedFragment>`. Precomposed visible-window replay is
deferred until overlay/preedit diagnostics need that broader contract.

## M2 - Prototype With Row-Level Fallback

Status: Complete

Implement the chosen shape without deleting the current row-level plan path. The prototype must
fall back on stale frame sequence, rect mismatch, missing row cache entry, overlay/preedit conflict,
or unsupported replay key.

Evidence:

- `row_scene_replay_plan_rejects_stale_frame_and_skipped_rows`
- `prepaint_row_scene_replay_plan_uses_cached_syntax_replay_context`
- `prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows`
- `planned_replay_rows_with_selection_still_paint_overlay`

## M3 - Perf Validation

Status: Complete

Run the complex-wheel local repro and generate `worst.stats.json`. A useful win should move p95
`us_row_scene_prepaint_probe` materially below the current `77us` without increasing frame paint or
renderer prepare enough to erase the gain.

Evidence:

- `target/fret-diag/local-next-editor-paint-20260516-retained-row-fragment-r2/worst.stats.json`
- repeat summary p95 `top_code_editor_row_scene_prepaint_probe_us`: `77 -> 40us`
- repeat summary p95 `top_code_editor_row_scene_prepaint_plan_us`: `95 -> 49us`
- repeat summary p95 `top_code_editor_windowed_surface_paint_callback_us`: `153 -> 120us`

## M4 - Close Or Promote

Status: Complete for local row-fragment scope

If the prototype wins, promote it as the default path and update `ui-perf-zed-smoothness-v1`. If it
does not win, close this lane with a no-ship audit and choose the next owner from fresh attribution.

Current decision: keep the retained-fragment prototype. The next owner in the same local repro is
not renderer text prepare; the worst r2 frame is dominated by semantics refresh/layout bookkeeping
outside row-fragment planning.

Post-prototype three-probe decision:

- typical autoscroll and complex wheel are no longer dominated by row-scene planning or renderer text
  prepare;
- resize jitter is dominated by layout roots / engine solve, with the top layout hotspot in the
  gallery content `ScrollArea`;
- any further row-fragment broadening should wait until resize/layout churn is reduced or fresh
  attribution makes row-fragment prepaint material again.
