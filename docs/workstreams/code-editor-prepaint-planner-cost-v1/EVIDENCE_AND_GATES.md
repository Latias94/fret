# Evidence And Gates

Date: 2026-05-15

## Focused Gate

```bash
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed (`3` tests).

## Package Gate

```bash
cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed (`130` tests).

## Check Gate

```bash
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-diag --features syntax-rust --all-targets
```

Result on 2026-05-15: passed.

## Format, Diff, And Boundary Gates

```bash
cargo fmt --check
git diff --check
python3 tools/check_layering.py
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
  --dir target/fret-diag/code-editor-prepaint-planner-cost-v1-after-fast-replay-context-counts-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Result on 2026-05-15: passed.

Worst bundle:

- `target/fret-diag/code-editor-prepaint-planner-cost-v1-after-fast-replay-context-counts-20260515/1778843491632/bundle.schema2.json`

## Bundle Comparison

Closed-lane baseline:

- bundle:
  `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515/1778841130928/bundle.schema2.json`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `91us`
- `code_editor_paint_perf.p95.total`: `1233us`
- `code_editor_paint_perf.p95.prepaint`: `353us`
- `rows_scene_fast_miss_no_entry`: `0`
- `rows_scene_full_miss_no_entry`: `0`

Current lane bundle:

- bundle:
  `target/fret-diag/code-editor-prepaint-planner-cost-v1-after-fast-replay-context-counts-20260515/1778843491632/bundle.schema2.json`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `67us`
- `code_editor_paint_perf.p95.total`: `1120us`
- `code_editor_paint_perf.p95.prepaint`: `278us`
- `rows_scene_fast_miss_no_entry`: `0`
- `rows_scene_full_miss_no_entry`: `0`
- `rows_scene_prepaint_edge_stored`: sum `5`, max `1`
- `rows_scene_prepaint_planned`: sum `2890`, max `289`
- `rows_scene_prepaint_plan_used`: sum `2890`, max `289`

Interpretation: the replay planner is now cheaper while preserving the paint miss invariants. If the
planner is still the dominant tail after the next slice, keep reducing it inside this lane; if the
dominant hotspot moves elsewhere, split a new owner lane instead of widening this one.
