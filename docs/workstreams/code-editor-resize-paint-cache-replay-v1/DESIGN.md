# Code Editor Resize Paint/Cache Replay V1

Date: 2026-05-15
Status: Closed after M1; use the closeout audit for the shipped verdict.

## Goal

Reduce live-resize tail cost on the `ui-code-editor-resize-probes` proof surface by tightening
internal row paint/cache boundaries in `ecosystem/fret-code-editor`.

This lane is deliberately narrow. The prior layout resize-measure audit showed the steady resize
tail is paint-dominant, not a broad layout dirty-frontier problem. This lane therefore keeps
`crates/fret-ui` layout apply-skip, text prepare, view-cache containment, `Scroll`, and
`VirtualList` semantics unchanged unless new evidence contradicts that stance.

## Baseline Attribution

Primary baseline bundle:

- `target/fret-diag/code-editor-resize-paint-cache-replay-v1-baseline-20260515-r2/1778821617964/bundle.schema2.json`

Baseline `diag stats --sort time --top 15 --json` summary:

- total p50/p95/max: `1287/1642/1642us`
- layout p50/p95/max: `34/356/356us`
- prepaint p50/p95/max: `335/382/382us`
- paint p50/p95/max: `764/956/956us`
- paint.widget p50/p95: `561/748us`
- `code_editor_paint_perf.p95.us_row_content_resolve=351us`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan=134us`
- `code_editor_paint_perf.p95.us_row_scene_replay_ops=38us`
- `code_editor_paint_perf.p95.us_row_scene_replay_touch=37us`
- `code_editor_paint_perf.p95.us_syntax_spans=12us`
- `code_editor_paint_perf.p95.us_row_rich_cache_compare=26us`
- `code_editor_paint_perf.p95.rows_scene_replayed=289`
- `code_editor_paint_perf.p95.rows_scene_prepaint_planned=289`

Interpretation: row scene replay hit rate is already high. The next useful work is not "make the
cache hit"; it is reducing the per-row work that remains after replay has already been proven.

## Architecture Decision

Keep the first slice inside `paint_row` and `scene` ownership:

- A replay-plan hit owns enough row content and geometry snapshot data to draw cached text and
  overlays for that row.
- Once a replay-plan hit is consumed, paint should not re-enter syntax-span lookup or rich-row
  content probing for that same row.
- Row geom cache maintenance stays in the existing geometry cache owner so pointer hit-testing and
  IME anchoring remain correct.

The intended direction is an incremental owner split, not an API expansion:

- `paint/mod.rs`: orchestration and overlay draw order.
- `paint/scene.rs`: row scene replay plan, fast-path validation, store/replay.
- `paint/geom_cache.rs`: row geometry cache freshness and LRU maintenance.
- `paint/text.rs` and `paint/rich.rs`: content materialization and rich cache inputs.

## First Slice

M1 short-circuits the syntax/rich content path after a replay-plan row scene hit:

- changed `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- extended `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`

The focused test now asserts that planned replay rows do not redo row text work, syntax-span lookup,
or rich-row cache probing in paint.

M1 after-run evidence:

- after worst bundle:
  `target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515/1778822452927/bundle.schema2.json`
- total p95: `1642us` -> `1469us`
- paint p95: `956us` -> `848us`
- paint.widget p95: `748us` -> `652us`
- `code_editor_paint_perf.p95.us_total`: `444us` -> `361us`
- `code_editor_paint_perf.p95.us_row_content_resolve`: `351us` -> `283us`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `134us` -> `83us`
- `code_editor_paint_perf.p95.us_syntax_spans`: `12us` -> `7us`
- `code_editor_paint_perf.p95.us_row_rich_cache_compare`: `26us` -> `15us`

The remaining dominant code-editor-owned cost is still row content resolve, so the next slice should
continue inside row snapshot/cache ownership before considering any view-cache or runtime change.

## Non-Goals

- Do not broaden `fret-ui` runtime contracts.
- Do not hide layout cost with another global apply-skip.
- Do not weaken code editor syntax, preedit, caret, selection, or hit-test geometry correctness.
- Do not convert this lane into broad editor feature work.
