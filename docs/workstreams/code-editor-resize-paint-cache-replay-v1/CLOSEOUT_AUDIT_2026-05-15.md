# Closeout Audit

Date: 2026-05-15
Status: Closed

## Objective

Complete `code-editor resize paint/cache replay fearless refactor v1`:

- use `ui-code-editor-resize-probes` as the primary proof surface,
- run with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`,
- attribute and reduce live-resize `paint.widget`, row content resolve, and row scene replay tail
  cost,
- keep the change inside `ecosystem/fret-code-editor` row content / row geom / row scene cache
  boundaries,
- avoid broad `fret-ui` layout apply-skip or semantic changes to text prepare, layout solve,
  view-cache containment, `Scroll`, or `VirtualList`,
- close with diag perf evidence, `code_editor_paint_perf` attribution, focused correctness gates,
  and workstream docs.

## Shipped Change

- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
  - Planned replay rows now skip the syntax/rich content path after row scene replay has already
    succeeded.
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
  - The replay-plan regression test now asserts no row text, syntax-span, or rich-row cache probe is
    redone in paint for planned replay rows.

No `crates/fret-ui` layout, text prepare, view-cache, scroll, or virtual-list files were changed.

## Evidence

Baseline:

- command: see `EVIDENCE_AND_GATES.md`
- worst bundle:
  `target/fret-diag/code-editor-resize-paint-cache-replay-v1-baseline-20260515-r2/1778821617964/bundle.schema2.json`

After M1:

- command: same proof surface and env, with `--dir target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515`
- worst bundle:
  `target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515/1778822452927/bundle.schema2.json`

Perf comparison:

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

## Gates

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --features syntax-rust --no-fail-fast`
  - Result: passed (`1` test).
- `cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast`
  - Result: passed (`129` tests).
- `cargo check -p fret-code-editor --features syntax-rust --all-targets`
  - Result: passed.
- `cargo fmt --check`
  - Result: passed.
- `git diff --check`
  - Result: passed.

## Prompt-to-Artifact Checklist

- Primary proof surface `ui-code-editor-resize-probes`: covered by baseline and after diag perf runs.
- Required env `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`: included in both recorded commands.
- Attribute `paint.widget`: baseline and after p95 values recorded.
- Attribute row content resolve: `code_editor_paint_perf.p95.us_row_content_resolve` recorded and
  reduced.
- Attribute row scene replay: replay op/touch p95 fields recorded and reduced.
- Scope stays in `ecosystem/fret-code-editor`: touched code is limited to code-editor paint and
  test files.
- Do not expand `fret-ui` layout apply-skip or semantics: no `crates/fret-ui` files changed.
- Focused correctness gates: replay-plan counter test and package nextest passed.
- Workstream docs: `WORKSTREAM.json`, `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `M1_REPLAY_PLAN_SHORT_PATH_2026-05-15.md`, and this closeout audit.

## Residual Risk And Follow-On

The remaining dominant code-editor-owned subfield is still row content resolve
(`283us` p95 after M1). If more resize paint budget is needed, start a new narrow follow-on for row
content snapshot/cache ownership rather than reopening broad `fret-ui` layout or scroll/view-cache
work.
