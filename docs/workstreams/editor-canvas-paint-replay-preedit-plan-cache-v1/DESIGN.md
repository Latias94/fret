# Editor Canvas Paint Replay Preedit Plan Cache v1

## Problem

The r61 plan-cache closeout proved the overlapping-window replay-plan cache works for stable resize/autoscroll, but
the complex-wheel/preedit-heavy scenario still reports:

- `plan_cache_hits=0`
- `rows_scene_prepaint_candidates=10115`
- `us_row_scene_prepaint_probe=2800`
- `us_row_scene_prepaint_key_compare=323`

The implementation still disables the replay-plan cache key when `st.preedit.is_some()`. That is wider than the
existing row-level paint contract. The row-level contract says only rows that actually require paint-time preedit
must stay off the prepaint replay path.

## Target State

- A visible frame with active preedit can still reuse cached replay-plan entries for rows not touched by preedit.
- Rows that require paint-time preedit remain excluded from the prepaint replay plan.
- No Canvas, renderer, or `WindowedRowsSurface` contract changes.
- Checked-in baselines remain unchanged until target-machine validation proves the effect.

## Scope

- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
- Workstream docs under `docs/workstreams/editor-canvas-paint-replay-preedit-plan-cache-v1/`

## Non-Goals

- No preedit rendering policy change.
- No Canvas display-list cache rewrite.
- No renderer text/glyph residency change.
- No baseline promotion from focused tests.

## Design

The replay-plan cache key should describe the stable frame and style context. It should not reject the whole frame
just because a preedit exists elsewhere. The planner already calls `row_requires_paint_time_preedit(st, row)` before
probing a row, so the correct boundary is row-level:

1. Build the cached plan lookup whenever the frame context is otherwise valid.
2. For each visible row, check whether the row requires paint-time preedit before using a cached plan entry.
3. Reuse cached retained fragments for non-preedit rows only when the current `row_scene_cache` still points at the
   same retained fragment.
4. Save any non-empty validated replay plan. Frames with a skipped preedit row can therefore preserve a partial cache
   for unrelated rows while the preedit row remains on the paint-time path.

This keeps the preedit correctness invariant while allowing unrelated rows to avoid repeated candidate probes and
key comparisons.
