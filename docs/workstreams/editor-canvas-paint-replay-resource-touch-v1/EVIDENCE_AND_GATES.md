# Editor Canvas Paint Replay Resource Touch v1 Evidence And Gates

## Starting Evidence

- r62 closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/editor-paint-contract-closeout.summary.json`
- r62 complex-wheel attribution:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib/runner-logs/complex-wheel/stats.stdout.json`

Key starting numbers:

- complex-wheel sum `plan_cache_hits=10041`
- complex-wheel sum `us_row_scene_replay_touch=1439`
- complex-wheel sum `us_row_scene_replay_ops=1179`
- complex-wheel p95 `code_editor_windowed_surface_row_paint=262us`
- owner remains `canvas-paint-replay`

## Local Gates

```powershell
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit row_scene_replay_plan_rejects_stale_frame_and_skipped_rows --features syntax-rust --no-fail-fast
cargo fmt -p fret-ui -p fret-code-editor --check
cargo check -p fret-code-editor --tests --features syntax-rust
python -m json.tool docs/workstreams/editor-canvas-paint-replay-resource-touch-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

Local evidence:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit row_scene_replay_plan_rejects_stale_frame_and_skipped_rows --features syntax-rust --no-fail-fast`
  passed on 2026-05-24.
- `cargo fmt -p fret-ui -p fret-code-editor --check` passed on 2026-05-24.
- `cargo check -p fret-code-editor --tests --features syntax-rust` passed on 2026-05-24 with only
  the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
- `python -m json.tool docs/workstreams/editor-canvas-paint-replay-resource-touch-v1/WORKSTREAM.json`
  passed on 2026-05-24.
- `python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json` passed on
  2026-05-24.
- `python tools/check_workstream_catalog.py` passed on 2026-05-24 with `433` dedicated directories
  and `47` standalone markdown files.
- `git diff --check` passed on 2026-05-24; it reported only a working-copy line-ending warning for
  `Cargo.lock`.

## Target-Machine Gates

```powershell
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r63-resource-touch-baseline --keep-going
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r63-resource-touch-attrib --with-paint-perf --keep-going
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-attrib
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-attrib --out-report target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline/editor-paint-contract-closeout.summary.json
```

Target-machine evidence:

- First baseline attempt:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline/summary.json`
  completed but failed `typical-autoscroll` once with
  `frame_p95_total_time_us=4229us` over the effective `3460us` threshold.
- Immediate standalone `typical-autoscroll` rerun:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-typical-rerun/check.perf_thresholds.json`
  passed with `0` failures and worst top total `1965us`.
- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/summary.json`
  passed on 2026-05-24.
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-attrib-rerun/summary.json`
  passed on 2026-05-24.
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/artifact-verification.summary.json`
  passed on 2026-05-24.
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/editor-paint-contract-closeout.summary.json`
  passed on 2026-05-24.

Key attribution result:

- resize-jitter moved from r62 `touch_p95/sum=59/431us`, `row_paint_p95=404us` to r63
  `44/415us`, `row_paint_p95=254us`.
- typical-autoscroll moved from r62 `touch_p95/sum=65/9109us` to r63 `58/8736us`; row paint stayed
  roughly flat (`318us -> 327us` p95).
- complex-wheel stayed Canvas-replay-owned: r63 reports `paint_widget_p95=516us`,
  `canvas_exclusive_p95=370us`, `code_editor_total_p95=314us`, and
  `us_row_scene_replay_touch` `p95/sum=63/1610us`.
- Closeout still selects `owner=canvas-paint-replay`; no checked-in baseline changes.

## Baseline Policy

No checked-in perf baseline changes are allowed from focused unit tests or local attribution alone.
Use the target-machine closeout to decide whether this slice changes the next owner or only becomes
a baseline-neutral cleanup.
