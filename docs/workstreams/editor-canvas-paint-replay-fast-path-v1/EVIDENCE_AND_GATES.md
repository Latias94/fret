# Editor Canvas Paint Replay Fast Path v1 Evidence And Gates

## Starting Evidence

- r64 closeout audit:
  `docs/workstreams/editor-canvas-paint-replay-row-setup-v1/CLOSEOUT_AUDIT_2026-05-24.md`
- r64 baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/summary.json`
- r64 rebuilt attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt/summary.json`
- r64 closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/editor-paint-contract-closeout.summary.json`

Key starting result:

- typical-autoscroll: `setup_p95/sum=62/9418us`, `touch_p95/sum=57/7798us`,
  `ops_p95/sum=83/12960us`, `row_paint_p95/sum=295/47555us`.
- complex-wheel: `setup_p95/sum=44/1280us`, `touch_p95/sum=53/1516us`,
  `ops_p95/sum=45/1194us`, `row_paint_p95/sum=272/7531us`.

## Local Gates

```powershell
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit planned_replay_rows_with_selection_still_paint_overlay retained_row_scene_origin_preserves_bounds_offset --features syntax-rust --no-fail-fast
cargo check -p fret-code-editor --tests --features syntax-rust
cargo check -p fret-code-editor --tests
cargo fmt -p fret-code-editor --check
python -m json.tool docs/workstreams/editor-canvas-paint-replay-fast-path-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

Local evidence:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit planned_replay_rows_with_selection_still_paint_overlay retained_row_scene_origin_preserves_bounds_offset --features syntax-rust --no-fail-fast`
  passed on 2026-05-24 with only the pre-existing `fret-ui` `current_effective_opacity`
  dead-code warning. The planned replay movement test clears `baseline_measure_cache` before the
  replay frame and asserts `us_baseline_measure == 0`, proving the no-overlay fast path bypasses
  baseline measurement rather than merely hitting a warm cache.
- `cargo check -p fret-code-editor --tests --features syntax-rust` passed on 2026-05-24 with the
  same pre-existing warning.
- `cargo check -p fret-code-editor --tests` passed on 2026-05-24. This verifies the non-syntax
  `store_row_scene_cache` signature path; it reports pre-existing non-syntax dead-code warnings.
- `cargo fmt -p fret-code-editor --check` passed on 2026-05-24.
- `python -m json.tool docs/workstreams/editor-canvas-paint-replay-fast-path-v1/WORKSTREAM.json`
  passed on 2026-05-24.
- `python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json` passed on
  2026-05-24.
- `python tools/check_workstream_catalog.py` passed on 2026-05-24 with `435` dedicated directories
  and `47` standalone markdown files.
- `git diff --check` passed on 2026-05-24.

## Target-Machine Gates

```powershell
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r65-row-fast-path-baseline --keep-going
cargo build -p fretboard-dev -p fret-ui-gallery --release --features fret-ui-gallery/gallery-full
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r65-row-fast-path-attrib --with-paint-perf --keep-going
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib --out-report target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/editor-paint-contract-closeout.summary.json
```

## Target-Machine Results

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/editor-paint-contract-closeout.summary.json`
- Typical-autoscroll stats:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib/runner-logs/typical-autoscroll/stats.stdout.json`
- Complex-wheel stats:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib/runner-logs/complex-wheel/stats.stdout.json`

Key results:

- typical-autoscroll: `setup_p95/sum=30/4368us`, `touch_p95/sum=57/8350us`,
  `ops_p95/sum=70/10651us`, `row_paint_p95/sum=250/40632us`, `total_p95/sum=227/37011us`.
- complex-wheel: `setup_p95/sum=15/442us`, `touch_p95/sum=39/983us`,
  `ops_p95/sum=28/1005us`, `row_paint_p95/sum=327/5186us`, `total_p95/sum=313/4688us`.

## Baseline Policy

This lane is closed. It reduced code-editor paint overhead without editing checked-in baselines, and
the target-machine closeout kept the baseline unchanged. Any follow-on work should use a new lane.
