# Milestones

## M0 - Baseline Carry-Over

Status: Done by closed-lane evidence

Baseline source:

- `docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/CLOSEOUT_AUDIT_2026-05-15.md`
- worst bundle:
  `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515/1778841130928/bundle.schema2.json`

Exit criteria:

- The closed lane stays closed.
- This lane owns only planner-cost reduction work.

## M1 - Cached Replay Context Fast-Path

Status: Done on 2026-05-15

Shipped:

- `RowSceneSyntaxReplayKey` can validate the cached replay context without re-looking up syntax
  spans.
- The replay planner uses that cached replay context for syntax rows and keeps plain-row replay
  planning unchanged.
- A focused regression test proves the planner trusts cached replay context.

Evidence:

- `ecosystem/fret-code-editor/src/editor/mod.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
- `target/fret-diag/code-editor-prepaint-planner-cost-v1-after-fast-replay-context-counts-20260515/1778843491632/bundle.schema2.json`

## M2 - Further Planner Reduction

Status: Closed on 2026-05-16

Exit criteria:

- `us_row_scene_prepaint_plan` falls below the current closed-lane baseline by a meaningful margin
  without regressing paint misses.
- the resize probe still reports `rows_scene_fast_miss_no_entry == 0` and
  `rows_scene_full_miss_no_entry == 0`.

Evidence:

- `target/fret-diag/code-editor-prepaint-planner-cost-v1-after-fast-replay-context-counts-20260515/1778843491632/bundle.schema2.json`
- `docs/workstreams/code-editor-prepaint-planner-cost-v1/CLOSEOUT_AUDIT_2026-05-16.md`

## Closeout

This lane is closed. The cached replay-context fast path reduced the measured planner p95 from the
closed-lane baseline and preserved the paint miss invariants. Fresh post-merge evidence shows the
dominant editor perf work has moved to paint/widget and Canvas paint/cache replay attribution, so
future work should continue in `ui-perf-zed-smoothness-v1` P1.5 or a narrower follow-on owner lane
instead of reopening this folder.
