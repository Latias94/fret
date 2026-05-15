# Code Editor Row Content Snapshot Cache V1

Date: 2026-05-15
Status: Closed after M2; use the closeout audit for the shipped verdict.

## Goal

Make row content a clear cache-owned payload instead of a loose tuple repeatedly moved through
prepaint and paint.

This lane follows `code-editor-resize-paint-cache-replay-v1`. That earlier lane proved row scene
replay is already highly effective during `ui-code-editor-resize-probes`; the remaining useful work
is reducing the per-row content bookkeeping that survives replay, not widening global layout or
view-cache contracts.

## Baseline Position

Prior after-run bundle:

- `target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515/1778822452927/bundle.schema2.json`

Relevant p95 values:

- total: `1469us`
- paint: `848us`
- paint.widget: `652us`
- `code_editor_paint_perf.us_total`: `361us`
- `code_editor_paint_perf.us_row_content_resolve`: `283us`
- `code_editor_paint_perf.us_row_scene_prepaint_plan`: `83us`

Interpretation: the row scene cache hit rate is not the main issue. The row content payload needed
a stronger owner so replay planning and paint could reuse it without calling row text materializers
or passing range/text/fold/span tuples around.

## Architecture Decision

The row content snapshot is now the payload owner for:

- displayed row text,
- buffer range,
- row fold map,
- preedit range,
- display row spans.

Ownership shape:

- `paint/text.rs` materializes `Arc<RowContentSnapshot>`.
- `CodeEditorState.row_text_cache` stores `Arc<RowContentSnapshot>`.
- `RowSceneCacheEntry` stores the same `Arc<RowContentSnapshot>`.
- `RowSceneFragmentPayload` carries the `Arc<RowContentSnapshot>` into paint.
- `paint_row` derives local read-only variables from the snapshot only after resolving whether the
  row came from a replay plan or row text cache.

This keeps component policy and runtime semantics unchanged. The change is local to
`ecosystem/fret-code-editor`.

## Non-Goals

- Do not change `crates/fret-ui` layout/view-cache semantics.
- Do not change `Scroll`, `VirtualList`, text prepare, or global paint cache behavior.
- Do not batch low-level scene replay ops in this lane.
- Do not change syntax, preedit, caret, selection, hit-testing, or accessibility semantics.

## Residual Shape

The normal replay-hit path now has low `row_content_resolve` in repeated runs, but the visible tail
can still spike when an edge row misses and takes the full row content/rich path. That should be a
separate follow-on focused on edge-row full-path work, not a broad architecture rewrite.
