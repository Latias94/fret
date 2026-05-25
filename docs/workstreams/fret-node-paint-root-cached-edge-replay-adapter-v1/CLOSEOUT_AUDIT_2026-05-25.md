# Fret Node Paint Root Cached Edge Replay Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the cached edge and edge-label replay adapter seam without widening into edge build-state
route inputs, temporary scene construction, cache keys, overlays, or public scene schema changes.

## Shipped State

- `cached_edges/replay_adapter.rs` defines `PaintRootCachedEdgeReplayCx` and the cached edge replay
  scene sink contract.
- `cached_edges/replay_retained_cx.rs` is the retained `PaintCx` binding for cached edge replay
  scene access.
- `cached_edges/edges/replay.rs` no longer mentions `PaintCx` or direct `cx.scene` access.
- `cached_edges/labels/replay.rs` no longer mentions `PaintCx` or direct `cx.scene` access.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the adapter free of retained
  lifecycle context names and scene ops, verifies both replay helper files use the adapter, and
  verifies the retained binding owns retained scene access.

## Split State

The following cached edge operation families remain intentionally outside this lane:

- edge build-state host/services/scale route inputs,
- edge-label build-state host/services/scale route inputs,
- temporary `fret_core::Scene` construction,
- cache-local clip-op emission,
- cache key semantics,
- overlay routing.

The next follow-on should choose build-state route inputs for cached edges or edge labels. Those
paths still read retained `cx.app`, `cx.services`, and `cx.scale_factor` directly.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-cached-static-scene-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/replay.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/replay.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/replay_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/replay_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-replay-adapter-v1/WORKSTREAM.json` -
  passed.
- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_replay_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risks

- Cached edge and edge-label build-state paths still read retained host/services/scale fields
  directly.
- Cache-local temporary scene and clip-op construction remains in cached edge build paths.
