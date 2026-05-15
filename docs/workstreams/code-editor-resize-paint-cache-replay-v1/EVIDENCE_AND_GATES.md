# Evidence And Gates

## Baseline Repro

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
  --dir target/fret-diag/code-editor-resize-paint-cache-replay-v1-baseline-20260515-r2 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Baseline worst bundle:

- `target/fret-diag/code-editor-resize-paint-cache-replay-v1-baseline-20260515-r2/1778821617964/bundle.schema2.json`

## M1 Gates

Focused gate:

```bash
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed (`1` test).

Package gate:

```bash
cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed (`129` tests).

Format gate:

```bash
cargo fmt --check
```

Result on 2026-05-15: passed.

Check gate:

```bash
cargo check -p fret-code-editor --features syntax-rust --all-targets
```

Result on 2026-05-15: passed.

## After-Change Perf Repro

Use the same command shape as the baseline, changing only `--dir`:

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
  --dir target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Result on 2026-05-15: passed.

After worst bundle:

- `target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515/1778822452927/bundle.schema2.json`

Aggregate comparison:

- total p95: `1642us` -> `1469us`
- paint p95: `956us` -> `848us`
- paint.widget p95: `748us` -> `652us`
- `code_editor_paint_perf.p95.us_total`: `444us` -> `361us`
- `code_editor_paint_perf.p95.us_row_content_resolve`: `351us` -> `283us`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `134us` -> `83us`
- `code_editor_paint_perf.p95.us_row_scene_replay_ops`: `38us` -> `26us`
- `code_editor_paint_perf.p95.us_row_scene_replay_touch`: `37us` -> `23us`
