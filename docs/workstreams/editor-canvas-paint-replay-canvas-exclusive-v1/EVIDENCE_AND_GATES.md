# Editor Canvas Paint Replay Canvas Exclusive v1 Evidence And Gates

## Starting Evidence

- Fast-path closeout audit:
  `docs/workstreams/editor-canvas-paint-replay-fast-path-v1/CLOSEOUT_AUDIT_2026-05-24.md`
- r65 baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/summary.json`
- r65 rebuilt attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib/summary.json`
- r65 closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/editor-paint-contract-closeout.summary.json`
- Source-backed attribution split:
  `crates/fret-ui/src/declarative/host_widget/paint.rs`,
  `crates/fret-ui/src/tree/debug/frame_stats.rs`,
  `crates/fret-ui/src/tree/paint/entry.rs`,
  `crates/fret-ui/src/tree/ui_tree_debug/paint.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/frame_stats.rs`,
  `crates/fret-diag/src/perf_keys.rs`

Key starting result:

- typical-autoscroll: `paint_widget=400`, `canvas=291`, `renderer_prepare_text=73`,
  `renderer_encode_scene=282`, `renderer_upload=335`, `code_editor_total=227`
- complex-wheel: `paint_widget=452`, `canvas=348`, `renderer_prepare_text=72`,
  `renderer_encode_scene=208`, `renderer_upload=332`, `code_editor_total=313`

## Completed Local Verification

```powershell
cargo fmt -p fret-ui -p fret-bootstrap -p fret-diag --check
cargo check -p fret-ui --tests
cargo check -p fret-bootstrap
cargo check -p fret-diag
cargo nextest run -p fret-diag full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_units_match_names trace_exported_perf_key_registry_contains_core_timeline_keys --no-fail-fast
git diff --check
```

Observed result:

- `paint_canvas_on_paint_time_us` is registered in `fret-diag`, and the perf-key registry tests
  passed.

## Local Gates

```powershell
rg -n "debug_paint_widget_exclusive|debug_paint_widget_hotspots|CanvasPainter|on_paint" crates\fret-ui ecosystem\fret-code-editor -g "*.rs"
cargo check -p fret-ui --tests
cargo check -p fret-code-editor --tests --features syntax-rust
python -m json.tool docs/workstreams/editor-canvas-paint-replay-canvas-exclusive-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

## Target-Machine Gates

```powershell
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r66-canvas-exclusive-baseline --keep-going
cargo build -p fretboard-dev -p fret-ui-gallery --release --features fret-ui-gallery/gallery-full
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r66-canvas-exclusive-attrib --with-paint-perf --keep-going
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260524-r66-canvas-exclusive-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260524-r66-canvas-exclusive-attrib
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260524-r66-canvas-exclusive-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260524-r66-canvas-exclusive-attrib --out-report target/fret-diag/editor-paint-contract-validate-20260524-r66-canvas-exclusive-baseline/editor-paint-contract-closeout.summary.json
```

## Baseline Policy

This lane is allowed to reduce Canvas exclusive / `paint.widget` overhead, but it must not edit
checked-in baselines until target-machine validation and closeout prove that a change is justified.
