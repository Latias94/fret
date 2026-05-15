# Evidence And Gates

## Focused Gate

```bash
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed (`1` test).

## Package Gate

```bash
cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed (`129` tests).

## Check Gate

```bash
cargo check -p fret-code-editor --features syntax-rust --all-targets
```

Result on 2026-05-15: passed.

## Perf Repro

```bash
target/release/fretboard-dev diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --dir target/fret-diag/code-editor-row-content-snapshot-cache-v1-after-m2-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Result on 2026-05-15: passed.

Worst bundle:

- `target/fret-diag/code-editor-row-content-snapshot-cache-v1-after-m2-20260515/1778827921081/bundle.schema2.json`

Aggregate p95:

- total: `1418us`
- paint: `866us`
- prepaint: `347us`

Worst-bundle `code_editor_paint_perf.p95`:

- `us_total`: `394us`
- `us_row_content_resolve`: `305us`
- `us_row_scene_prepaint_plan`: `70us`
- `us_row_scene_replay_ops`: `26us`
- `us_row_scene_replay_touch`: `22us`
- `us_row_text`: `12us`
- `us_row_rich_cache_compare`: `23us`
- `us_row_geom_key`: `55us`

Per-run row content p95 from the same perf run:

- `1778827912180`: `110us`
- `1778827916039`: `116us`
- `1778827921081`: `305us`

Interpretation: stable replay-hit rows now reuse the snapshot cheaply; the remaining tail is from an
edge-row full path.
