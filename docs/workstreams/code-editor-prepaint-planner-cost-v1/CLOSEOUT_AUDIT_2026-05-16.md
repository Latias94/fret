# Code Editor Prepaint Planner Cost v1 Closeout Audit

Date: 2026-05-16
Status: Closed

## Objective

Close the narrow prepaint-planner follow-on after verifying whether code-editor replay planning is
still the dominant editor performance bottleneck.

This lane only owned reducing `us_row_scene_prepaint_plan` after the closed
`code-editor-edge-row-full-path-prefetch-v1` lane. It did not own Canvas paint replay,
renderer payload, display-list architecture, or VirtualList ownership.

## Evidence

Closed-lane baseline:

- Bundle:
  `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515/1778841130928/bundle.schema2.json`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `91us`
- `code_editor_paint_perf.p95.total`: `1233us`
- `code_editor_paint_perf.p95.prepaint`: `353us`
- `rows_scene_fast_miss_no_entry`: `0`
- `rows_scene_full_miss_no_entry`: `0`

Current lane bundle:

- Bundle:
  `target/fret-diag/code-editor-prepaint-planner-cost-v1-after-fast-replay-context-counts-20260515/1778843491632/bundle.schema2.json`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `67us`
- `code_editor_paint_perf.p95.total`: `1120us`
- `code_editor_paint_perf.p95.prepaint`: `278us`
- `rows_scene_fast_miss_no_entry`: `0`
- `rows_scene_full_miss_no_entry`: `0`
- `rows_scene_prepaint_planned`: sum `2890`, max `289`
- `rows_scene_prepaint_plan_used`: sum `2890`, max `289`

Post-merge direction evidence:

- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry
  `2026-05-15 20:21:29 +08:00` reports the current macOS typical autoscroll sample as
  paint/widget dominated, not layout, VirtualList, cache-miss, or row-scene planner dominated.
- The same entry reports row replay hit rate `100%`,
  rows painted/scene-replayed/scene-stored=`289/289/0`, and code-editor paint p50/p95 total
  `126/149us`.

## Gates

The lane's gates were already recorded in
`docs/workstreams/code-editor-prepaint-planner-cost-v1/EVIDENCE_AND_GATES.md`:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast`
- `cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-diag --features syntax-rust --all-targets`
- `cargo fmt --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- the `ui-code-editor-resize-probes` perf repro with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`

## Verdict

Close this lane.

The cached replay-context fast path materially reduced the prepaint planner cost while preserving
the paint miss invariants. Fresh evidence then moved the dominant editor performance question away
from prepaint planning. Continuing to reduce `us_row_scene_prepaint_plan` now has weak expected
value compared with closing Editor Canvas paint/cache replay attribution.

## Follow-On

Continue in `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-todo.md` P1.5
(`Editor canvas paint replay`).

The next slice should verify the current row-scene replay short-circuit and then attribute any
remaining `paint.widget` / Canvas cost across:

- planned row replay paint short-circuit,
- row content resolve,
- row-scene replay/cache replay,
- renderer encode/upload payload.

If that evidence points to renderer encode/upload, split a renderer owner lane. If it points to
generic Canvas/display-list ownership, split a narrow Canvas owner lane. Do not reopen this
prepaint-planner folder unless a future bundle again proves replay planning is the dominant tail.
