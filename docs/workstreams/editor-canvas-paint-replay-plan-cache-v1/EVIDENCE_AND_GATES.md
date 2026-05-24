# Editor Canvas Paint Replay Plan Cache v1 Evidence and Gates

## Seed Evidence

- Previous lane closeout:
  `docs/workstreams/editor-canvas-paint-replay-slice-v1/CLOSEOUT_AUDIT_2026-05-23.md`
- r59 baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`
- r59 attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib/summary.json`

## Implementation Evidence

- Replay plan cache and invalidation:
  `ecosystem/fret-code-editor/src/editor/state.rs`
- Stable-window reuse path:
  `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- Buffer replacement cache invalidation:
  `ecosystem/fret-code-editor/src/editor/handle/model.rs`
- Paint perf diagnostics:
  `ecosystem/fret-code-editor/src/editor/diagnostics.rs`
- Diagnostics JSON export:
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
- `fretboard-dev diag stats` aggregation:
  `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs`,
  `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
- Regression test:
  `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`

## Required Gates

```powershell
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast
cargo check -p fret-code-editor --tests --features syntax-rust
cargo fmt -p fret-code-editor -p fret-diag -p fret-ui-gallery --check
git diff --check
python -m json.tool docs/workstreams/editor-canvas-paint-replay-plan-cache-v1/WORKSTREAM.json
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
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast
cargo check -p fret-code-editor --tests --features syntax-rust
cargo fmt -p fret-code-editor -p fret-diag -p fret-ui-gallery --check
git diff --check
```

Target-machine passed:

```powershell
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r61-plan-cache-baseline --keep-going
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r61-plan-cache-attrib --with-paint-perf --keep-going
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib --out-report target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/editor-paint-contract-closeout.summary.json
```

Refreshed stats were regenerated after rebuilding `target/release/fretboard-dev.exe`:

```powershell
cargo build -p fretboard-dev --release
target\release\fretboard-dev.exe diag stats target\fret-diag\editor-paint-contract-validate-20260523-r61-plan-cache-attrib\resize-jitter\attempt-1\1779542327454\bundle.schema2.json --sort cpu_cycles --top 15 --json
target\release\fretboard-dev.exe diag stats target\fret-diag\editor-paint-contract-validate-20260523-r61-plan-cache-attrib\typical-autoscroll\1779542958224\bundle.schema2.json --sort cpu_cycles --top 15 --json
target\release\fretboard-dev.exe diag stats target\fret-diag\editor-paint-contract-validate-20260523-r61-plan-cache-attrib\complex-wheel\1779543210418\bundle.schema2.json --sort cpu_cycles --top 15 --json
```

Plan-cache evidence from refreshed stats:

- resize-jitter: frames `10`, sum `plan_cache_hits=2885`, `plan_cache_rejects=0`, `candidates=5`,
  `planned=2890`, `probe=0us`, `key_compare=0us`.
- typical-autoscroll: frames `180`, sum `plan_cache_hits=51930`, `plan_cache_rejects=0`, `candidates=90`,
  `planned=52020`, `probe=0us`, `key_compare=0us`.
- complex-wheel: frames `35`, sum `plan_cache_hits=0`, `plan_cache_rejects=0`, `candidates=10115`,
  `planned=10076`, `probe=2800us`, `key_compare=323us`.

Closeout decision:

- `owner=canvas-paint-replay`
- `action=open-canvas-paint-replay-slice`
- Baseline policy unchanged.

Known warning:

- `cargo check` still reports the pre-existing `fret-ui` dead-code warning for
  `current_effective_opacity`.
