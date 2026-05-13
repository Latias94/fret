# Evidence and Gates

Status: Active
Last updated: 2026-05-13

## Primary Repro

The first repro remains the code-editor resize/paint pressure path:

```bash
cargo run -p fretboard-dev --release -- diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Use an explicit `--dir target/<descriptive-dir>` for publishable evidence.

## Required Attribution

For every perf claim, run:

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 15
```

The summary must mention:

- total/layout/prepaint/paint p50 and p95,
- `layout.engine_solve`,
- `paint.widget`,
- `paint.text_prepare`,
- renderer prepare/encode/upload counters,
- and `code_editor.paint_perf` when the code-editor surface is involved.

## Current Baseline Evidence

Most recent pre-lane evidence:

- M0 baseline/source audit:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M0_BASELINE_AUDIT_2026-05-13.md`
- M1 boundary diagnostics slice:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M1_BOUNDARY_DIAGNOSTICS_SLICE_2026-05-13.md`
- Workstream log:
  `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- macOS contained-layout run:
  `target/fret-diag-code-editor-resize-probes-contained-layout-20260513/check.perf_thresholds.json`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-contained-layout-20260513/1778661520873/bundle.schema2.json`

Observed result from that run:

- gate failures: `[]`,
- p95/max top total: `1361/1361us`,
- p95/max top layout: `295/295us`,
- p95/max top layout solve: `116/116us`,
- p95/max paint: `1134/1134us`,
- `code_editor.paint_perf` p50/p95 total: `241/401us`.

Most recent boundary-diagnostics slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M1_BOUNDARY_DIAGNOSTICS_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-boundary-diag-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-boundary-diag-20260513/1778668519515/bundle.schema2.json`

Observed result from that run:

- time p50/p95: total `1203/1811us`, layout `38/364us`, prepaint `15/34us`,
  paint `949/1737us`,
- hot p50/p95: `layout.engine_solve=0/140us`, `paint.widget=731/1494us`,
  `paint.text_prepare=10/15us`,
- `code_editor.paint_perf` p50/p95 total: `302/743us`,
- renderer prepare/encode/upload counters stayed at zero.

Most recent prepaint-ownership slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M2_CANVAS_PREPAINT_WINDOWED_ROWS_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-20260513/1778671598958/bundle.schema2.json`

Observed result from that run:

- time p50/p95: total `1117/1356us`, layout `33/335us`, prepaint `168/291us`,
  paint `722/897us`,
- hot p50/p95: `layout.engine_solve=0/129us`, `paint.widget=523/695us`,
  `paint.text_prepare=9/12us`,
- `code_editor.paint_perf` p50/p95 total: `261/433us`,
- `code_editor.paint_perf.us_frame_overlay` sum: `0`,
- row scene replay hit rate: `99%`,
- renderer prepare/encode/upload counters stayed at zero.

This slice moved editor frame-derived ownership out of paint attribution and into prepaint, but it
did not yet finish the full boundary migration or the final 20-30% end-to-end improvement target.

Most recent row-scene prepaint replay-plan slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M3_ROW_SCENE_PREPAINT_REPLAY_PLAN_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-row-scene-prepaint-plan-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-row-scene-prepaint-plan-20260513/1778679317011/bundle.schema2.json`

Observed result from that run:

- gate failures: `[]`,
- total p50/p95/max: `1443/1712/1712us`,
- layout p50/p95/max: `387/388/388us`,
- prepaint p50/p95/max: `282/382/382us`,
- paint p50/p95/max: `814/943/943us`,
- row scene replay hit rate: `99-100%`,
- renderer prepare/encode/upload counters stayed at `0`.

Worst-bundle attribution:

- time p50/p95: total `1170/1712`, layout `37/387`, prepaint `324/382`, paint `710/958`
- hot p50/p95: `layout.engine_solve=0/146`, `paint.widget=499/745`,
  `paint.text_prepare=10/12`
- `code_editor.paint_perf` sum planned/used replay entries: `2090/2090`
- max planned/used replay entries per frame: `289/289`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `65/123us`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/6us`

This slice proves the planned phase split for cached row scene replay. It does not yet complete the
final `ViewBoundary` store or the final 20-30% end-to-end improvement target.

Most recent windowed-rows canonical row-rect slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M3A_WINDOWED_ROWS_CANONICAL_ROW_RECT_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-windowed-row-rect-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-windowed-row-rect-20260513/1778681710195/bundle.schema2.json`

Observed result from that run:

- gate failures: `[]`,
- total p50/p95/max: `1250/1519/1519us`,
- layout p50/p95/max: `335/345/345us`,
- prepaint p50/p95/max: `275/349/349us`,
- paint p50/p95/max: `727/825/825us`,
- row scene replay hit rate: `99%`,
- renderer prepare/encode/upload counters stayed at `0`.

Worst-bundle attribution:

- time p50/p95: total `1125/1519`, layout `35/345`, prepaint `265/380`, paint `672/900`
- hot p50/p95: `layout.engine_solve=0/127`, `paint.widget=456/691`,
  `paint.text_prepare=9/12`
- `code_editor.paint_perf` sum planned/used replay entries: `2090/2090`
- max planned/used replay entries per frame: `289/289`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `67/89us`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/12us`

This slice removes code-editor-local fixed-row rect reconstruction from replay planning. It keeps
the replay plan editor-owned, so it is still a transition step before the final boundary fragment
store.

Most recent canvas prepaint-output slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M3B_ROW_SCENE_PREPAINT_OUTPUT_CARRIER_SLICE_2026-05-13.md`
- Focused correctness gate:
  `cargo nextest run -p fret-ui declarative::tests::canvas::canvas_prepaint_output_is_visible_to_canvas_paint --no-fail-fast`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513/1778685213875/bundle.schema2.json`

Observed result from that run:

- time p50/p95/max: total `1103/1576/1576us`,
  layout `35/344/344us`,
  prepaint `251/360/360us`,
  paint `659/877/877us`
- hot p50/p95: `layout.engine_solve=0/133us`, `paint.widget=445/661us`,
  `paint.text_prepare=10/13us`
- `code_editor.paint_perf` p50/p95 total: `175/403us`
- `code_editor.paint_perf.us_row_text` p50/p95: `0/5us`
- planned/used replay entries still matched `2090/2090`
- row scene replay hit rate remained `99%`
- renderer prepare/encode/upload counters stayed at zero

Worst-bundle attribution:

- `target/release/fretboard-dev diag stats target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513/1778685213875/bundle.schema2.json --sort time --top 15`
- time p50/p95: total `1103/1576us`, layout `35/344us`, prepaint `251/360us`,
  paint `659/877us`
- hot p50/p95: `layout.engine_solve=0/133us`, `paint.widget=445/661us`,
  `paint.text_prepare=10/13us`
- `code_editor.paint_perf` sum planned/used replay entries: `2090/2090`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `55/77us`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/5us`

## Correctness Gates

Use focused tests first:

```bash
cargo nextest run -p fret-ui <filter>
cargo test -p fret-ui-gallery --features gallery-full --lib <filter>
cargo test -p fret-ui-shadcn --lib <filter>
```

Required for boundary/invalidation changes:

```bash
python3 tools/check_layering.py
```

## Future Paint Stressor

If `ui-code-editor-resize-probes` stops catching the active paint bottleneck, add a narrower
code-editor paint stressor before continuing:

- route directly to the code-editor torture surface,
- keep `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`,
- stress row replay/content resolution without unrelated gallery setup noise,
- and seed a baseline/policy only after the script is deterministic.

## Closeout Evidence

Closeout requires:

- final perf run paths,
- final worst-bundle attribution,
- deletion audit path,
- ADR alignment row,
- and exact commands for all promoted gates.
