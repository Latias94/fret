# Evidence And Gates

Date: 2026-05-15

## Focused Gate

```bash
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed (`2` tests).

## Package Gate

```bash
cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed (`130` tests).

## Check Gate

```bash
cargo check -p fret-code-editor --features syntax-rust --all-targets
```

Result on 2026-05-15: passed.

## Format And Diff Gates

```bash
cargo fmt --check
git diff --check
```

Result on 2026-05-15: both passed.

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
  --dir target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m1-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Result on 2026-05-15: passed.

Worst bundle:

- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m1-20260515/1778830062977/bundle.schema2.json`

Aggregate p95 from the perf run:

- total: `1442us`
- paint: `747us`
- prepaint: `328us`
- layout: `376us`

Worst-bundle p95:

- total: `1442us`
- paint: `873us`
- paint.widget: `669us`
- prepaint: `328us`
- layout: `376us`

Worst-bundle `code_editor_paint_perf.p95` after M1:

- `us_total`: `371us`
- `us_row_content_resolve`: `281us`
- `us_row_scene_prepaint_plan`: `111us`
- `us_row_scene_replay_ops`: `31us`
- `us_row_scene_replay_touch`: `39us`
- `us_row_text`: `6us`
- `us_row_rich_cache_compare`: `20us`
- `us_row_geom_key`: `53us`
- `rows_scene_prepaint_planned`: `289`
- `rows_scene_prepaint_plan_used`: `289`
- `rows_scene_replayed`: `289`

Comparison against previous row-content snapshot worst bundle:

- previous bundle:
  `target/fret-diag/code-editor-row-content-snapshot-cache-v1-after-m2-20260515/1778827921081/bundle.schema2.json`
- `code_editor_paint_perf.p95.us_total`: `394us` -> `371us`
- `us_row_content_resolve`: `305us` -> `281us`
- `us_row_text`: `12us` -> `6us`
- `us_row_rich_cache_compare`: `23us` -> `20us`
- `us_row_geom_key`: `55us` -> `53us`
- `us_row_scene_prepaint_plan`: `70us` -> `111us`
- worst-bundle paint p95: `912us` -> `873us`
- worst-bundle prepaint p95: `347us` -> `328us`
- worst-bundle total p95: `1418us` -> `1442us`

Interpretation: M1 improves the code-editor-owned paint path and increases replay-plan coverage,
but it is not a full edge-row payload prebuild. The next slice should make candidate planning
cheaper and more edge-aware before adding broader prebuild mechanics.

## M2 Diagnostics

Additional validation for the diagnostics slice:

```bash
cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast
```

Result on 2026-05-15: passed.

Perf repro:

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
  --dir target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m2-diagnostics-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Worst bundle:

- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m2-diagnostics-20260515/1778832028679/bundle.schema2.json`

Aggregate p95 from the M2 diagnostics perf run:

- total: `1336us`
- paint: `753us`
- prepaint: `241us`
- layout: `342us`

Worst-bundle `code_editor_paint_perf.p95` after M2 diagnostics:

- `us_total`: `313us`
- `us_row_content_resolve`: `210us`
- `us_row_scene_prepaint_plan`: `90us`
- `rows_scene_prepaint_candidates`: `289`
- `rows_scene_prepaint_planned`: `289`
- `rows_scene_prepaint_skip_key_mismatch`: `72`
- `rows_scene_prepaint_skip_no_cache`: `1`
- `rows_scene_fast_miss_no_entry`: `1`
- `rows_scene_full_miss_no_entry`: `1`
- `rows_scene_stored_at_visible_end`: `1`
- `rows_scene_stored_at_visible_start`: `0`

Interpretation: the remaining paint full miss is a no-cache row at the newly exposed visible end.
Key-mismatch skips are also meaningful because they make prepaint planning do work that paint can
often recover from later. Continue with code-editor-local planner-cost reduction and visible-end row
seeding; do not start a broad `fret-ui` architecture refactor from this evidence.

## M2 Follow-Up Syntax Replay Key Equality

Focused validation:

```bash
cargo nextest run -p fret-code-editor syntax_replay_key_matches_equivalent_current_inputs --features syntax-rust --no-fail-fast
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast
cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed.

Perf repro:

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
  --dir target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-syntax-key-content-eq-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Worst bundle:

- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-syntax-key-content-eq-20260515/1778835965902/bundle.schema2.json`

Aggregate p95 from the follow-up perf run:

- total: `1270us`
- paint: `729us`
- prepaint: `255us`
- layout: `354us`

Worst-bundle `code_editor_paint_perf.p95` after the follow-up:

- `us_total`: `191us`
- `us_row_content_resolve`: `116us`
- `us_row_scene_prepaint_plan`: `75us`
- `rows_scene_prepaint_skip_key_mismatch`: `0`
- `rows_scene_prepaint_skip_no_cache`: `1`
- `rows_scene_fast_miss_no_entry`: `1`
- `rows_scene_full_miss_no_entry`: `1`
- `rows_scene_stored_at_visible_end`: `1`

Interpretation: syntax key content equality removed the planner key-mismatch tail, but the goal is
still open because the newly exposed visible-end row still reaches paint without a row scene cache
entry.

## M3 Edge Row Payload Prebuild

Focused validation:

```bash
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast
cargo nextest run -p fret-ui declarative::tests::canvas::canvas_prepaint_can_prepare_text_scene_fragment_before_paint --no-fail-fast
cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-diag --features syntax-rust --all-targets
cargo fmt --check
git diff --check
python3 tools/check_layering.py
python3 -m json.tool docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/WORKSTREAM.json
```

Result on 2026-05-15: passed.

Perf repro:

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
  --dir target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Worst bundle:

- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515/1778841130928/bundle.schema2.json`

Aggregate p95 from the M3 perf run:

- total: `1233us`
- paint: `664us`
- prepaint: `353us`
- layout: `326us`

Worst-bundle `code_editor_paint_perf.p95` after M3:

- `rows_scene_prepaint_edge_stored`: `1`
- `row_scene_prepaint_edge_ops_stored`: `1`
- `rows_scene_prepaint_skip_no_cache`: `0`
- `rows_scene_fast_miss_no_entry`: `0`
- `rows_scene_full_miss_no_entry`: `0`
- `rows_scene_stored`: `0`
- `rows_scene_stored_at_visible_end`: `0`
- `rows_scene_replayed`: `289`
- `rows_scene_prepaint_planned`: `289`
- `rows_scene_prepaint_plan_used`: `289`
- `us_row_text`: `0`
- `us_row_scene_store`: `0`
- `us_row_scene_full_path`: `0`
- `us_total`: `163us`

Worst-bundle snapshot distribution:

- `rows_scene_prepaint_edge_stored`: sum `5`, max `1`
- `row_scene_prepaint_edge_ops_stored`: sum `5`, max `1`
- `rows_scene_stored`: sum `0`, max `0`
- `rows_scene_stored_at_visible_end`: sum `0`, max `0`
- `rows_scene_prepaint_skip_no_cache`: sum `0`, max `0`
- `rows_scene_fast_miss_no_entry`: sum `0`, max `0`
- `rows_scene_full_miss_no_entry`: sum `0`, max `0`

Interpretation: M3 completes the lane objective. New edge rows are prepared during prepaint, replay
planning covers the full visible window, and paint no longer performs the no-entry full row scene
path.
