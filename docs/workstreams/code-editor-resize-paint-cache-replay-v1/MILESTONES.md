# Code Editor Resize Paint/Cache Replay Milestones

## M0 - Baseline Attribution

Status: Done

Done criteria:

- `ui-code-editor-resize-probes` runs with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`.
- Worst bundle is recorded.
- `diag stats` identifies the top row paint/cache subfields.

Evidence:

- Baseline bundle:
  `target/fret-diag/code-editor-resize-paint-cache-replay-v1-baseline-20260515-r2/1778821617964/bundle.schema2.json`

## M1 - Replay-Plan Paint Short Path

Status: Done

Done criteria:

- Planned replay rows do not redo row text, syntax-span lookup, or rich-row cache probing in paint.
- Focused `fret-code-editor` test covers the counter contract.
- After-change perf run is captured and compared.

Touched code:

- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`

Evidence:

- After bundle:
  `target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515/1778822452927/bundle.schema2.json`
- `code_editor_paint_perf.p95.us_total`: `444us` -> `361us`
- `code_editor_paint_perf.p95.us_row_content_resolve`: `351us` -> `283us`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `134us` -> `83us`

## M2 - Next Narrow Bottleneck

Status: Recommended follow-on

Candidate targets, decided only after M1 after-run evidence:

- row content snapshot/cache ownership, because it remains the dominant code-editor p95 subfield,
- then row geom cache touch/write overhead if row content resolve no longer dominates,
- then row scene replay op/touch overhead,
- view-cache root rerender ownership only if a later bundle shows it is the dominant remaining
  cause.

## Closeout

Status: Done for v1; future resize paint budget should start as a narrow follow-on for row content
snapshot/cache ownership.

Close the lane only after:

- after-run evidence exists,
- focused gates pass,
- residual bottleneck and follow-on guidance are recorded.
