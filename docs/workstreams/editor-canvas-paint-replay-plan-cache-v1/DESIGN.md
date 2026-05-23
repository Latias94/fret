# Editor Canvas Paint Replay Plan Cache v1

## Problem

The previous Canvas replay lane closed after `20260523-r59`, but the closeout still retained
`owner=canvas-paint-replay`. The fine-grained paint perf counters continued to show visible per-frame row-scene
planning work:

- resize-jitter: `prepaint_plan/probe/key_compare/replay_touch/replay_ops = 261/190/52/102/139us`
- typical-autoscroll: `203/147/35/71/101us`
- complex-wheel: `193/127/31/70/49us`

The repeated shape is stable or mostly-overlapping-window planning across roughly the same visible row set. That
makes the next bounded owner the code-editor row-scene replay planner, not generic Canvas traversal, renderer text
prepare, or WindowedRowsSurface iteration.

## Target State

- A validated row-scene replay plan can be reused for the same or overlapping visible row window when the frame
  context, style, theme, and row-scene cache contents remain valid.
- Reuse skips per-row candidate probing and key comparison for retained rows already present in the cached plan.
- The paint path still consumes the plan through the existing `frame_seq` and `local_bounds` checks.
- Whole-cache invalidation clears the replay-plan cache; ordinary row replacement/eviction is guarded by retained
  fragment identity checks before reuse.
- Checked-in baselines remain unchanged until a target-machine validation proves the effect.

## Scope

- `ecosystem/fret-code-editor/src/editor/state.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/handle/model.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`

## Non-Goals

- No Canvas display-list cache rewrite.
- No renderer text/glyph residency changes.
- No `fret-ui-kit::WindowedRowsSurface` contract change.
- No checked-in baseline promotion from local/focused test evidence.

## Design

The implementation adds a syntax-build row-scene replay plan cache to `CodeEditorState`.

The cache key includes:

- buffer/display-map/fold/inlay/feature-payload epochs,
- row count, row geometry, content x/width/height, and cache capacity,
- text style, text constraints, font stack, scale, theme revision, code font policy revision, and foreground color.

The planner first asks whether the current frame matches the cached context key. If yes, it builds a row-indexed
view of the previous retained plan. For every current visible row, it reuses the cached retained fragment only when
the current `row_scene_cache` still points at the same `Arc<RowSceneRetainedFragment>`. That identity check makes the
plan safe across sliding visible windows and across row replacement or eviction.

Rows that are not covered by a valid cached fragment fall back to the existing validated per-row probe path. The
planner saves any non-empty validated plan, so later frames can reuse overlapping or partial row windows. Whole-cache
clears and syntax replay-key refreshes clear the replay-plan cache directly.

## Verification Notes

The focused test `prepaint_row_scene_replay_plan_reuses_stable_window_plan` seeds replayable row-scene cache entries,
runs the validation planner once, then reruns the same frame and an overlapping shifted frame. The reuse pass must
report:

- `rows_scene_prepaint_candidates == 0`
- `rows_scene_prepaint_plan_cache_hits > 0`
- `us_row_scene_prepaint_probe == 0`
- `us_row_scene_prepaint_key_compare == 0`
- no synthetic `row_scene_fast_get_calls`

This proves the mechanism-level optimization. Target-machine perf validation is still required before baseline policy
changes.

## Closeout Result

The 2026-05-23 r61 target-machine validation closed this lane as a baseline-neutral mechanism improvement. It showed
strong replay-plan cache reuse for resize-jitter and typical-autoscroll, but not for the complex-wheel/preedit-heavy
scenario. The checked-in baselines remain unchanged, and the parent performance lane keeps `canvas-paint-replay` as
the next owner rather than treating this narrow cache as the full Canvas replay closeout.
