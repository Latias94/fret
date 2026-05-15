# M1 Replay-Plan Paint Short Path

Date: 2026-05-15
Status: Closed slice; lane can continue with a row content snapshot/cache follow-on.

## Objective

Reduce live-resize code-editor paint work by making a planned row scene replay hit stop the syntax
and rich-content portion of `paint_row`.

## Change

- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
  - The syntax/rich content branch now runs only when row scene replay has not already succeeded.
  - A prepaint replay-plan hit still preserves cached row content and geometry payloads for overlay
    drawing and row-geometry cache maintenance.
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
  - The replay-plan test now asserts that paint does not redo row text work, syntax-span lookup, or
    rich-row cache probing for planned replay rows.

## Gates

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --features syntax-rust --no-fail-fast`
  - Result: passed (`1` test).
- `cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast`
  - Result: passed (`129` tests).
- `cargo check -p fret-code-editor --features syntax-rust --all-targets`
  - Result: passed.
- `cargo fmt --check`
  - Result: passed.

## Perf Evidence

Baseline bundle:

- `target/fret-diag/code-editor-resize-paint-cache-replay-v1-baseline-20260515-r2/1778821617964/bundle.schema2.json`

After bundle:

- `target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515/1778822452927/bundle.schema2.json`

Comparison:

- total p95: `1642us` -> `1469us`
- paint p95: `956us` -> `848us`
- paint.widget p95: `748us` -> `652us`
- `code_editor_paint_perf.p95.us_total`: `444us` -> `361us`
- `code_editor_paint_perf.p95.us_row_content_resolve`: `351us` -> `283us`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `134us` -> `83us`
- `code_editor_paint_perf.p95.us_row_scene_replay_ops`: `38us` -> `26us`
- `code_editor_paint_perf.p95.us_row_scene_replay_touch`: `37us` -> `23us`
- `code_editor_paint_perf.p95.us_syntax_spans`: `12us` -> `7us`
- `code_editor_paint_perf.p95.us_row_rich_cache_compare`: `26us` -> `15us`

## Verdict

M1 validates the narrow internal direction: reducing work after replay has already been proven
improves the resize proof surface without touching runtime layout, text prepare, view-cache
containment, or scroll/virtual-list semantics.

The next high-value target is still inside `ecosystem/fret-code-editor`: row content
snapshot/cache ownership remains the dominant code-editor p95 subfield after M1.
