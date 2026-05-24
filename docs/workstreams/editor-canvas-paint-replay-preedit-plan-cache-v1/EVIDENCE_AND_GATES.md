# Editor Canvas Paint Replay Preedit Plan Cache v1 Evidence and Gates

## Seed Evidence

- r61 plan-cache closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/editor-paint-contract-closeout.summary.json`
- r61 complex-wheel stats:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib/runner-logs/complex-wheel/stats.stdout.json`
- Previous lane closeout:
  `docs/workstreams/editor-canvas-paint-replay-plan-cache-v1/CLOSEOUT_AUDIT_2026-05-23.md`

## Required Gates

```powershell
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows prepaint_row_scene_replay_plan_reuses_stable_window_plan --features syntax-rust --no-fail-fast
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --tests --features syntax-rust
git diff --check
python -m json.tool docs/workstreams/editor-canvas-paint-replay-preedit-plan-cache-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
```

## Target-Machine Gate

Before any baseline policy change:

```powershell
python tools/perf/diag_editor_paint_contract_validate.py --date-tag <date> --with-paint-perf
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-<date> --attribution-dir target/fret-diag/editor-paint-contract-validate-<date>-attrib
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-<date> --attribution-dir target/fret-diag/editor-paint-contract-validate-<date>-attrib
```

## Current Verification

Date: 2026-05-23

Passed:

```powershell
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows prepaint_row_scene_replay_plan_reuses_stable_window_plan --features syntax-rust --no-fail-fast
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --tests --features syntax-rust
python -m json.tool docs/workstreams/editor-canvas-paint-replay-preedit-plan-cache-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r62-preedit-plan-cache-baseline --keep-going
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r62-preedit-plan-cache-attrib --with-paint-perf --keep-going
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib --out-report target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/editor-paint-contract-closeout.summary.json
```

What it proves:

- `prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit` seeds retained row-scene entries,
  marks row `0` as the active preedit row, saves a partial replay plan for row `1`, then proves the next frame reuses
  row `1` through `rows_scene_prepaint_plan_cache_hits` while row `0` is still counted as
  `rows_scene_prepaint_skip_preedit`.
- Existing inline preedit row-skip and stable-window replay-plan cache tests still pass.

Known warning:

- `cargo nextest` still reports the pre-existing `fret-ui` dead-code warning for
  `current_effective_opacity`.
- `cargo check` reports the same pre-existing warning.

## Target-Machine Result

The r62 validation passed:

- `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/summary.json`
- `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib/summary.json`
- `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/artifact-verification.summary.json`
- `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/editor-paint-contract-closeout.summary.json`

Refreshed stats:

- resize-jitter: frames `10`, sum `plan_cache_hits=2885`, `candidates=5`, `probe=0us`, `key_compare=0us`.
- typical-autoscroll: frames `180`, sum `plan_cache_hits=51930`, `candidates=90`, `probe=0us`,
  `key_compare=0us`.
- complex-wheel: frames `35`, sum `plan_cache_hits=10041`, `candidates=74`, `skip_preedit=35`, `probe=7us`,
  `key_compare=0us`.

Closeout decision:

- `owner=canvas-paint-replay`
- `action=open-canvas-paint-replay-slice`
- Baseline policy unchanged.
