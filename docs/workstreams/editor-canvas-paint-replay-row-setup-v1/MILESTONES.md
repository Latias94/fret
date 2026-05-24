# Editor Canvas Paint Replay Row Setup v1 Milestones

## M0 - Lane Open

Status: complete.

Exit criteria:

- Workstream docs exist and point at the r63 owner decision.
- Parent performance lane records this as the next bounded Canvas replay/row-paint follow-on.

Evidence:

- `docs/workstreams/editor-canvas-paint-replay-resource-touch-v1/CLOSEOUT_AUDIT_2026-05-24.md`
- `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/editor-paint-contract-closeout.summary.json`
- `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-attrib-rerun/runner-logs/complex-wheel/stats.stdout.json`

## M1 - Planned Replay Setup Attribution

Status: complete.

Exit criteria:

- `CodeEditorPaintPerfFrame` includes us/ns planned replay setup counters.
- `paint_row` records setup time before matching planned replay enters scene replay.
- Gallery snapshots and `fret-diag stats` expose the counter.
- Focused local tests/checks pass.

## M2 - Local Gate Set

Status: complete.

Exit criteria:

- Focused `fret-diag` nextest passes.
- `fret-code-editor` and `fret-diag` test checks pass.
- Formatting, JSON, catalog, and diff gates pass.

Evidence:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
- `cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast`
- `cargo check -p fret-code-editor --tests --features syntax-rust`
- `cargo check -p fret-diag --tests`
- `cargo fmt -p fret-code-editor -p fret-diag -p fret-ui-gallery --check`
- `python -m json.tool docs/workstreams/editor-canvas-paint-replay-row-setup-v1/WORKSTREAM.json`
- `python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## M3 - Target-Machine Attribution

Status: complete.

Exit criteria:

- Baseline validation passes on the Windows RTX4090 target machine.
- Attribution validation with paint perf passes.
- Artifact verifier passes.
- Closeout names the next owner with the new replay setup metric included in the evidence.

Evidence:

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/summary.json`
- Rebuilt attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/editor-paint-contract-closeout.summary.json`

Result:

- Closeout still selects `owner=canvas-paint-replay`.
- Checked-in baseline policy remains unchanged.
