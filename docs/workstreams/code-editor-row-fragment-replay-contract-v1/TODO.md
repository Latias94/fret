# Code Editor Row Fragment Replay Contract v1 TODO

Status: Active
Date: 2026-05-16

## Current Evidence

- Local attribution:
  `target/fret-diag/local-next-editor-paint-20260516-prepaint-probe-attrib-complex-wheel-r3/worst.stats.json`
- Non-landed micro-cleanup:
  `target/fret-diag/local-next-editor-paint-20260516-prepaint-plan-small-opt-complex-wheel-r3/worst.stats.json`
- Current decision:
  - do not optimize renderer text/glyph residency from current evidence,
  - do not continue HashMap lookup micro-changes,
  - prototype a row-fragment replay contract.

## Checklist

- [x] Split prepaint plan cost into probe and key-compare attribution.
- [x] Test and reject the single mutable lookup / plan preallocation micro-cleanup.
- [x] Design the first row-fragment replay shape.
  - Candidate A: contiguous run descriptor that still points at per-row cached fragments but validates the run once.
  - Candidate B: precomposed visible-window scene fragment that replays all retained row ops before row paint.
  - Required decision: how overlay/preedit rows are represented without repainting base rows twice.
  - Decision: start with a conservative Candidate A precursor: plan entries point at retained per-row fragments to cut
    assembly clones while preserving per-row validation/fallback. Coarser run validation remains a follow-on.
- [x] Confirm diagnostics before changing behavior.
  - Required fields: fragment rows covered, rows used, rows skipped for overlay/preedit, stale fragment reason.
  - Result: the retained-fragment prototype uses existing row-scene planned/used/skip counters plus boundary
    scene-fragment used/rejected diagnostics. No schema change was needed for this slice because fallback still
    occurs through the existing row-level plan path.
- [x] Implement the smallest prototype behind the existing row-level fallback.
  - Preserve `RowSceneReplayPlan` fallback until the new path proves itself.
  - Preserve hosted-resource touch semantics before replaying retained scene ops.
  - Preserve row geom cache behavior for selection/caret overlay.
- [x] Add focused tests.
  - Planned full-fragment rows skip row text work.
  - Overlay/preedit rows still paint their overlays.
  - Stale frame and skipped-row entries reject the fragment and fall back; rect mismatch continues through the
    existing paint-level boundary rejection path.
- [x] Run local complex-wheel perf and compare against:
  - `us_row_scene_prepaint_probe` p95 `77us`,
  - `us_row_scene_prepaint_plan` p95 `95us`,
  - frame total p95 `808us`.
- [x] Update `ui-perf-zed-smoothness-v1` with the result.
  - If the prototype wins, keep the lane active for cleanup.
  - If it does not win, record closeout and choose the next owner from fresh attribution.

## Latest Local Result

- Prototype: retained per-row scene fragment references in `RowSceneReplayPlanEntry`, plus delayed
  geometry cloning for no-overlay planned rows.
- Evidence:
  `target/fret-diag/local-next-editor-paint-20260516-retained-row-fragment-r2/worst.stats.json`
- Complex-wheel repeat summary:
  - `top_code_editor_row_scene_prepaint_probe_us` p95: `77 -> 40us`
  - `top_code_editor_row_scene_prepaint_plan_us` p95: `95 -> 49us`
  - `top_code_editor_windowed_surface_paint_callback_us` p95: `153 -> 120us`
  - `top_renderer_prepare_text_us` p95: `37us`
- Worst total was dominated by a non-row-fragment frame: `total=935us`, `layout=439us`,
  `layout_semantics_refresh_time_us=399us`.

## Follow-On Candidates

- Investigate why `RunnerMonitorTopologyDiagnosticsStore` global changes and semantics refresh can
  keep the gallery shell root in `needs_rerender` during the complex-wheel script.
- If row-fragment prepaint remains material after that, prototype an exact visible-window plan cache
  over retained fragments. Do not jump to a precomposed visible-window scene until overlay/preedit
  diagnostics are stronger.

## Guardrails

- Do not change checked-in perf baselines from local macOS evidence.
- Do not mark the editor paint contract closeout goal complete from this lane.
- Do not widen `fret-ui` runtime policy; `fret-ui` owns the carrier/mechanism, code-editor owns row
  replay policy.
