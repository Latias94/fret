# M3 Row Scene Prepaint Replay Plan Slice - 2026-05-13

Status: landed; transitional row-scene replay plans move resize-frame cache validation out of paint.

## Scope

This slice keeps the code-editor row scene cache editor-owned, but moves the resize-frame replay
decision into the prepaint phase:

- prepaint builds a per-frame `RowSceneReplayPlan` from existing row scene cache entries,
- paint consumes the plan by row and rect before falling back to paint-side row text/cache lookup,
- diagnostics report planned vs used replay entries and the prepaint planning cost,
- `diag stats` prints the new counters in both JSON and human-readable summaries.

This is intentionally not the final `ViewBoundary` scene-fragment store. It is a narrow vertical
slice that proves the phase split before moving ownership into the runtime boundary model.

## Implementation

Main code-editor changes:

- `ecosystem/fret-code-editor/src/editor/state.rs`
  - `RowSceneCacheEntry.ops` now stores `Arc<[SceneOp]>` so prepaint plans and paint replay can share
    cached scene ops without cloning the op vector.
  - added `RowSceneReplayPlan` and `RowSceneReplayPlanEntry`.
  - added frame-scoped plan reset/push/take helpers.
- `ecosystem/fret-code-editor/src/editor/mod.rs`
  - added a row-scene replay planning hook at the end of the windowed-rows prepaint hook chain.
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
  - added `prepaint_row_scene_replay_plan_for_frame(...)`.
  - `paint_row(...)` now consumes a matching prepaint plan entry and skips paint-side
    `cached_row_text_with_range(...)` work for that row.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
  - added syntax-aware replay-plan candidate validation in prepaint.
  - factored shared replay-key matching and replay delta helpers.
  - kept replay touch and replay ops timing separate so existing diagnostics semantics remain
    stable.

Diagnostics changes:

- `ecosystem/fret-code-editor/src/editor/diagnostics.rs`
  - added `rows_scene_prepaint_planned`, `rows_scene_prepaint_plan_used`,
    `us_row_scene_prepaint_plan`, and `ns_row_scene_prepaint_plan`.
- `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
  - exports the new code-editor paint-perf fields.
- `crates/fret-diag/src/stats*.rs`
  - parses, aggregates, emits JSON, and prints the new prepaint-plan fields.

## Evidence

Focused correctness gates run for this slice:

- `cargo nextest run -p fret-code-editor --features syntax-rust prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax-rust row_text_cache --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax-rust prefetch --no-fail-fast`
- `cargo nextest run -p fret-ui canvas_prepaint --no-fail-fast`
- `cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast`
- `cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-diag --features syntax-rust`
- `python3 tools/check_layering.py`

Perf gate run for this slice:

```bash
cargo run -p fretboard-dev --release -- diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --dir target/fret-diag-code-editor-resize-probes-row-scene-prepaint-plan-20260513 \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Perf output directory:

- `target/fret-diag-code-editor-resize-probes-row-scene-prepaint-plan-20260513`

Worst bundle:

- `target/fret-diag-code-editor-resize-probes-row-scene-prepaint-plan-20260513/1778679317011/bundle.schema2.json`

Threshold result:

- `check.perf_thresholds.json` failures: `[]`
- observed max top total: `1712us` against threshold `16308us`
- observed max top layout: `388us` against threshold `3432us`
- observed max top layout solve: `158us` against threshold `372us`

Perf run summary:

- total p50/p95/max: `1443/1712/1712us`
- layout p50/p95/max: `387/388/388us`
- prepaint p50/p95/max: `282/382/382us`
- paint p50/p95/max: `814/943/943us`
- row scene replay hit rate: `99-100%`
- renderer prepare/encode/upload counters stayed at `0`

Worst-bundle attribution:

- `target/release/fretboard-dev diag stats target/fret-diag-code-editor-resize-probes-row-scene-prepaint-plan-20260513/1778679317011/bundle.schema2.json --sort time --top 15`
- time p50/p95: total `1170/1712us`, layout `37/387us`, prepaint `324/382us`,
  paint `710/958us`
- hot p50/p95: `layout.engine_solve=0/146us`, `paint.widget=499/745us`,
  `paint.text_prepare=10/12us`
- `code_editor.paint_perf` planned and used replay entries matched:
  `sum.rows_scene_prepaint_planned=2090`,
  `sum.rows_scene_prepaint_plan_used=2090`,
  max planned/used per frame: `289/289`
- `code_editor.paint_perf` p50/p95:
  `us_row_scene_prepaint_plan=65/123us`,
  `us_row_text=0/6us`

The expected effect is present: paint no longer performs row text work for planned rows. The work is
now visible as prepaint planning cost, which is the right phase for the next boundary-owned fragment
store migration.

## Deletion Audit

What changed:

- resize-frame row scene replay validation can now happen before paint,
- paint can consume prevalidated row scene fragments without doing row text lookup for each planned
  row,
- row-scene op storage no longer requires a fresh `Vec<SceneOp>` clone when a prepaint plan points at
  cached ops.

What is still intentionally old or transitional:

- `RowSceneReplayPlan` is editor-owned state, not the final `ViewBoundary` fragment store.
- row rect reconstruction in prepaint assumes the current fixed-height windowed rows surface
  (`content_origin.y + row_h * row`, no row gap and no scroll margin).
- only syntax-replayable row scene cache entries are planned in this slice.
- cache rejection diagnostics still live in code-editor counters rather than canonical boundary
  fragment diagnostics.
- paint-side fallback replay paths remain for rows without a matching plan.

Follow-up deletion/narrowing target:

- move replay plans into boundary-owned scene-fragment state,
- let the windowed rows prepaint surface provide a canonical row rect iterator instead of
  reconstructing rects locally,
- merge `rows_scene_prepaint_*` into boundary fragment diagnostics or delete them after the
  boundary diagnostics cover the same evidence,
- delete editor-local plan storage once the final `ViewBoundary` store owns fragment validation and
  replay.
