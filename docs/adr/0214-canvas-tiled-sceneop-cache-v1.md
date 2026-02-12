# ADR 0214: Canvas Tiled SceneOp Cache (v1)

Status: Proposed
Scope: Ecosystem canvas infrastructure (`ecosystem/fret-canvas`) and retained canvas widgets that emit large quad/path scenes.

## Summary

Introduce a spatially-aware replay cache (`SceneOpTileCache`) for retained canvas paint that stays hot across pan/scroll by:

- recording scene operations per fixed-size tile in *tile-local coordinates*,
- replaying cached ops with a caller-provided translation delta,
- standardizing cache key construction so translation is never hashed into the content key,
- supporting incremental warmup via per-frame budgets and diagnostics.

## Context

Fret’s retained canvases (ADR 0128) frequently draw large, spatially-local scenes:

- node graphs (background grids, lots of nodes/wires),
- plots (heatmaps / histogram2d with many quads),
- charts (dense marks).

Today, the primary reuse mechanism for “static-ish” paint is whole-layer replay caching
(`SceneOpCache` / subtree replay; ADR 0055). This works well when the view is stable, but it
degenerates for common editor interactions:

- **pan/scroll** changes translation every frame, causing cache keys to miss if they include view
  bounds/translation,
- even when content is mostly unchanged, we end up rebuilding and re-recording a large op vector.

We want a policy-light substrate that enables high hit rates for pan/scroll without forcing all
canvases into a single data model or a second rendering system.

## Goals

- Improve pan/scroll performance for retained canvases by reusing most work across frames.
- Enable incremental adoption: tile only the heaviest layers first.
- Provide a mechanism-level API (no domain models, no interaction policy baked in).
- Make tuning practical via diagnostics (hits/misses, warmup budgets, entry counts).

## Non-goals

- A new renderer or a GPU-side tiling system.
- A universal caching story for all `SceneOp` variants and all resource lifetimes.
- Prescribing tile sizes, interaction maps, or culling policies globally.
- Replacing declarative element-tree aspirations (ADR 0128); this is a retained optimization.

## Decision

### 1) Add a spatially-aware replay cache: `SceneOpTileCache`

Introduce `SceneOpTileCache<K>` in `ecosystem/fret-canvas`:

- records `Vec<SceneOp>` per tile key `K`,
- replays cached ops with a caller-provided translation delta,
- supports simple age/budget pruning.

This cache is intended for “many quads/paths, spatially local, translation-heavy” layers where a
monolithic cache would miss too often.

### 2) Canonical keying guidance: separate content from translation

For tile caching to be effective, callers MUST treat “content” and “placement” as separate inputs:

- **tile-local ops** MUST be recorded without view translation (pan offsets, view min/max). A common
  pattern is to record ops relative to `tile_origin` and then replay with `replay_delta = tile_origin`.
- **content key** MUST exclude translation and include only content-stable inputs:
  - model revision / content hash,
  - zoom/scale-dependent parameters (scale bits, mip level),
  - style knobs that affect geometry (spacing, stroke width, colors),
  - tile size (or a version tag that implies tile size).
- **tile coordinate** MUST be added separately (do not hand-roll hashing in every widget).

To reduce boilerplate and prevent accidental translation hashing, `fret-canvas` provides:

- `TileCacheKeyBuilder` for composing a deterministic `base_key: u64` from content-stable fields,
- `tile_cache_key(base_key, TileCoord) -> u64` for combining with the tile coordinate.

This keeps cache hits high across pan/scroll while still invalidating correctly on zoom or content
changes.

### 3) Budgeted warmup and diagnostics are first-class

Tile caches are most effective when callers can warm tiles incrementally without frame spikes:

- Use `WorkBudget` (units-per-frame) to cap tile builds on cache misses.
- When work is skipped due to budget, request a redraw so warmup continues over subsequent frames.
- Degrade budgets while interacting (pan/drag/zoom) and raise them when idle to keep input latency
  stable without making the UI feel “incomplete” for long.

Diagnostics SHOULD be recorded so budgets can be tuned per widget and per layer. `fret-canvas`
exposes per-window cache snapshots and helper recording APIs via `CanvasCacheStatsRegistry`.

### 4) Relationship to retained vs declarative surfaces

This ADR is a retained-canvas optimization:

- It assumes a retained paint phase records a scene op stream that can be replayed.
- It is compatible with future declarative element-tree work as an *internal* optimization: a
  declarative surface can still record scene ops into tiles per frame if it chooses, but the public
  contract should not depend on this cache.

### 5) Keep `fret-canvas` policy-light and modular

`ecosystem/fret-canvas` remains a substrate crate (ADR 0128):

- provides generic infrastructure (`cache`, `view`, `scale`, `drag`, `spatial`, `text`, `budget`),
- does not define domain models (node graphs / plots / charts),
- keeps heavier integration optional behind features (e.g. `ui`).

## Consequences

Pros:

- High cache hit rates for pan/scroll in large retained canvases.
- Low-risk adoption: callers can incrementally tile only the heaviest layers (e.g. background grid,
  heatmap quads) without refactoring the whole widget.
- Clear ownership boundary: `fret-canvas` provides mechanisms; ecosystem canvases choose policies.

Cons / Risks:

- Tile caching can increase memory usage (many tiles retained); pruning must be tuned per widget.
- Cached `SceneOp`s may reference renderer-owned resources; callers must ensure resource lifetimes
  outlive cached ops (same caveat as `SceneOpCache`).
- Poor tile-size choices can lead to “wasted ops” (too much offscreen work) or excessive tile
  bookkeeping (too many tiny tiles).

## Keying Checklist (Practical)

- Do not hash `pan`, `view_min/max`, `viewport rect`, or other per-frame translation into the key.
- Do hash: model revision, zoom bits, style knobs that affect geometry, tile size/version.
- Record tile ops in tile-local coordinates and apply translation only at replay time.
- Prefer stable version tags (`"...tile.v1"`) so key evolution is explicit.

## Implementation Notes (Non-normative)

Initial integrations (evidence anchors):

- Tile cache implementation: `ecosystem/fret-canvas/src/cache/scene_op_tile_cache.rs`
- Key helpers: `ecosystem/fret-canvas/src/cache/scene_op_tile_cache.rs` (`TileCacheKeyBuilder`,
  `tile_cache_key`)
- Node graph background grid (quad-heavy) uses tile caching:
  `ecosystem/fret-node/src/ui/canvas/widget.rs`
- Plot heatmap / histogram2d quads use tile caching fast path:
  `ecosystem/fret-plot/src/retained/layers.rs`
  `ecosystem/fret-plot/src/retained/canvas/mod.rs`
- Cache/budget diagnostics plumbing: `ecosystem/fret-canvas/src/diagnostics.rs`
  `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

Recommended future follow-ups:

- Add common “tile size selection” guidance per layer type (world-space vs screen-space tiles).
- Consider a small helper for “budgeted tile warmup” loops (build order + budget + redraw policy)
  when more layers adopt tiling.

## References

- Canvas contract and placement: `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- Whole-layer replay caching: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- Render transform + hit testing: `docs/adr/0082-render-transform-hit-testing.md`
