# Editor Canvas Paint Replay Row Setup v1 Evidence And Gates

## Starting Evidence

- r63 closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/editor-paint-contract-closeout.summary.json`
- r63 complex-wheel attribution:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-attrib-rerun/runner-logs/complex-wheel/stats.stdout.json`
- r63 closeout audit:
  `docs/workstreams/editor-canvas-paint-replay-resource-touch-v1/CLOSEOUT_AUDIT_2026-05-24.md`

Key starting result:

- complex-wheel stayed Canvas-replay-owned with `paint_widget_p95=516us`,
  `canvas_exclusive_p95=370us`, `code_editor_total_p95=314us`,
  `row_paint_p95=335us`, `us_row_scene_replay_touch p95/sum=63/1610us`, and
  `us_row_scene_replay_ops p95/sum=46/1140us`.

## Local Gates

```powershell
cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast
cargo check -p fret-code-editor --tests --features syntax-rust
cargo check -p fret-diag --tests
cargo fmt -p fret-code-editor -p fret-diag -p fret-ui-gallery --check
python -m json.tool docs/workstreams/editor-canvas-paint-replay-row-setup-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

Local evidence:

- `cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast`
  passed on 2026-05-24.
- `cargo check -p fret-code-editor --tests --features syntax-rust` passed on 2026-05-24 with only
  the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
- `cargo check -p fret-diag --tests` passed on 2026-05-24.
- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
  passed on 2026-05-24 with only the pre-existing `fret-ui` `current_effective_opacity`
  dead-code warning.
- `cargo fmt -p fret-code-editor -p fret-diag -p fret-ui-gallery --check` passed on 2026-05-24.
- `python -m json.tool docs/workstreams/editor-canvas-paint-replay-row-setup-v1/WORKSTREAM.json`
  passed on 2026-05-24.
- `python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json` passed on
  2026-05-24.
- `python tools/check_workstream_catalog.py` passed on 2026-05-24 with `434` dedicated directories
  and `47` standalone markdown files.
- `git diff --check` passed on 2026-05-24.

## Target-Machine Gates

```powershell
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r64-row-setup-baseline --keep-going
cargo build -p fretboard-dev -p fret-ui-gallery --release --features fret-ui-gallery/gallery-full
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r64-row-setup-attrib-rebuilt --with-paint-perf --keep-going
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt --out-report target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/editor-paint-contract-closeout.summary.json
```

Target-machine evidence:

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/summary.json`
  passed on 2026-05-24.
- Rebuilt attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt/summary.json`
  passed on 2026-05-24.
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/artifact-verification.summary.json`
  passed on 2026-05-24.
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/editor-paint-contract-closeout.summary.json`
  passed on 2026-05-24.

Note: the first attribution run with tag `20260524-r64-row-setup-attrib` used an older
`target/release/fretboard-dev.exe` from 2026-05-23 and did not include the new schema `14` counter.
The final evidence uses `20260524-r64-row-setup-attrib-rebuilt`, after rebuilding release
`fretboard-dev` and `fret-ui-gallery`.

Key attribution result:

- typical-autoscroll: `setup_p95/sum=62/9418us`, `touch_p95/sum=57/7798us`,
  `ops_p95/sum=83/12960us`, `row_paint_p95/sum=295/47555us`.
- complex-wheel: `setup_p95/sum=44/1280us`, `touch_p95/sum=53/1516us`,
  `ops_p95/sum=45/1194us`, `row_paint_p95/sum=272/7531us`.
- Closeout still selects `owner=canvas-paint-replay`; no checked-in baseline changes.

## Baseline Policy

No checked-in perf baseline changes are allowed from this diagnostics-only slice. The new counter
exists to make the next implementation slice measurable.
