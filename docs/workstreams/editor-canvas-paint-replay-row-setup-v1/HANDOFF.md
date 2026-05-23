# Editor Canvas Paint Replay Row Setup v1 Handoff

Status: active
Updated: 2026-05-24

## Current State

This lane follows the closed r63 resource-touch lane. The r63 closeout still selects
`owner=canvas-paint-replay`, but the remaining row-paint gap is not fully explained by existing
sub-counters.

The first slice adds planned replay setup attribution:

- `CodeEditorPaintPerfFrame::us_row_scene_replay_setup`
- `CodeEditorPaintPerfFrame::ns_row_scene_replay_setup`
- app snapshot schema version `14`
- `fret-diag stats` extraction, aggregation, JSON, percentile, and human output support

## Next Action

Run target-machine attribution before any optimization:

```powershell
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r64-row-setup-baseline --keep-going
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r64-row-setup-attrib --with-paint-perf --keep-going
```

## Validation

Passed so far on 2026-05-24:

- `cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast`
- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
- `cargo check -p fret-code-editor --tests --features syntax-rust`
- `cargo check -p fret-diag --tests`
- `cargo fmt -p fret-code-editor -p fret-diag -p fret-ui-gallery --check`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## Risks

- This slice is diagnostics-only; do not use it to change checked-in baselines.
- Do not batch row replay ops or change overlay semantics in this lane.
- If target-machine evidence shows setup is not material, split the next implementation slice by
  the measured owner rather than extending this lane blindly.
