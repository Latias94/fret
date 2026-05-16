# Code Editor Row Fragment Replay Contract v1 Evidence And Gates

Status: Active
Date: 2026-05-16

## Baseline Evidence

Local no-4090 attribution:

- `target/fret-diag/local-next-editor-paint-20260516-prepaint-probe-attrib-complex-wheel-r3/worst.stats.json`
- frame p95 total/paint/prepaint/layout: `808/433/275/184us`
- code-editor paint p95 total/prepaint-plan/prepaint-probe/key-compare/surface:
  `113/95/77/7/153us`

Rejected micro-cleanup:

- `target/fret-diag/local-next-editor-paint-20260516-prepaint-plan-small-opt-complex-wheel-r3/worst.stats.json`
- code-editor paint p95 prepaint-plan: `95 -> 94us`
- code-editor paint p95 prepaint-probe: `77 -> 85us`
- frame total p95: `808 -> 829us`
- result: not kept

## Retained Fragment Prototype Evidence

Local no-4090 complex-wheel rerun:

- `target/fret-diag/local-next-editor-paint-20260516-retained-row-fragment-r2/worst.stats.json`
- worst bundle:
  `target/fret-diag/local-next-editor-paint-20260516-retained-row-fragment-r2/1778941023307/bundle.schema2.json`
- retained-fragment shape:
  `RowSceneReplayPlanEntry` now carries row/local-bounds plus `Arc<RowSceneRetainedFragment>`.
- focused tests:
  `cargo nextest run -p fret-code-editor row_scene_replay_plan_rejects_stale_frame_and_skipped_rows prepaint_row_scene_replay_plan_uses_cached_syntax_replay_context prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`

Result versus the attribution baseline:

- `top_code_editor_row_scene_prepaint_probe_us` p95: `77 -> 40us`
- `top_code_editor_row_scene_prepaint_plan_us` p95: `95 -> 49us`
- `top_code_editor_windowed_surface_paint_callback_us` p95: `153 -> 120us`
- worst-bundle code-editor p95 `us_total`: `113 -> 75us`
- worst-bundle code-editor p95 `us_windowed_surface_row_paint`: `134 -> 86us`
- renderer text prepare stayed below the row-fragment owner: repeat p95 `37us`

The worst total frame in the r2 run is not a row-fragment failure: `total=935us`, `layout=439us`,
and `layout_semantics_refresh_time_us=399us`. It also reports a changed
`RunnerMonitorTopologyDiagnosticsStore` global and a gallery shell view-cache root with
`reuse_reason=needs_rerender`.

## Correctness Gates

```bash
cargo fmt -p fret-code-editor -p fret-diag -p fret-ui-gallery --check
cargo nextest run -p fret-code-editor row_scene_replay_plan_rejects_stale_frame_and_skipped_rows prepaint_row_scene_replay_plan_uses_cached_syntax_replay_context prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast
cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot top_code_editor_row_scene_fields_compute_replay_rate perf_json_row_exports_top_code_editor_row_scene_fields perf_repeat_run_json_row_exports_top_code_editor_row_scene_fields perf_repeat_summary_json_row_summarizes_code_editor_row_scene_fields --no-fail-fast
python3 tools/check_workstream_catalog.py
git diff --check
```

## Local Perf Repro

```bash
cargo run -p fretboard-dev --release -- diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_A11Y_DISABLE=1 \
  --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --sort time \
  --top 15 \
  --json \
  --dir target/fret-diag/<dir> \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness
```

Worst-bundle stats:

```bash
cargo run -p fretboard-dev --release -- diag stats target/fret-diag/<dir>/<bundle>/bundle.schema2.json \
  --sort time \
  --top 15 \
  --json > target/fret-diag/<dir>/worst.stats.json
```

## Acceptance Threshold

This lane should not claim success unless:

- p95 `code_editor_paint_perf.us_row_scene_prepaint_probe` moves materially below `77us`,
- p95 `code_editor_paint_perf.us_row_scene_prepaint_plan` moves below `95us`,
- row replay hit rate stays high for retained rows,
- overlay/preedit rows retain correctness through focused tests,
- and no checked-in baseline is changed from local macOS evidence.
