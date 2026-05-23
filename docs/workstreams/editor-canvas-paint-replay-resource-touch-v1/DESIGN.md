# Editor Canvas Paint Replay Resource Touch v1

## Problem

The r62 preedit replay-plan-cache closeout removed most row-scene prepaint probing from the
complex-wheel path, but the parent `ui-perf-zed-smoothness-v1` owner is still
`canvas-paint-replay`. The remaining attribution is now concentrated in planned row replay:

- complex-wheel p95 `code_editor_windowed_surface_row_paint=262us`
- complex-wheel sum `us_row_scene_replay_touch=1439`
- complex-wheel sum `us_row_scene_replay_ops=1179`
- resize-jitter p95 `row_callback_gap=97us`

The hot planned-replay path touches hosted resources once per replayed row. In the high-stress
editor scenarios, each planned row usually references one retained text blob, so the current shape
does hundreds of small resource-touch calls per frame before replaying row ops.

## Target State

- Planned row-scene replay touches hosted resources once per replay plan rather than once per row.
- Row replay order, overlay handling, row geom caching, and preedit exclusion semantics remain
  unchanged.
- No global renderer, text backend, or `WindowedRowsSurface` contract change.
- Checked-in baselines remain unchanged until target-machine validation proves the effect.

## Scope

- `crates/fret-ui/src/canvas.rs`
- `ecosystem/fret-code-editor/src/editor/state.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
- Parent workstream accounting under `docs/workstreams/ui-perf-zed-smoothness-v1/`

## Non-Goals

- No batching of row replay ops into a single scene transform.
- No change to Canvas hosted-resource eviction policy.
- No renderer encode/upload change.
- No baseline promotion from local tests.

## Design

The planned replay path already has a prepaint-owned `RowSceneReplayPlan`. This plan is the right
scope for aggregating resources because every entry in the plan has already passed the cache key,
preedit, and retained-fragment validation gates.

The safe shape is:

1. Let `CanvasHostedResources` merge precomputed retained resource sets.
2. Store the aggregate on `RowSceneReplayPlan`.
3. When paint consumes the first planned entry, touch the aggregate once.
4. Replay each row's retained ops exactly as before, using the per-entry text blob index for scene
   replay.

This preserves per-row replay semantics while removing repeated cache-map touch calls from the hot
planned-replay path.
