# Fret Node Paint Root Cached Edge Build State Temp Scene Adapter v1 - Closeout Audit

Date: 2026-05-25
Status: Closed

## Closeout Claim

Cached edge and edge-label build-state route helpers no longer construct cache-local temporary
scenes directly. Temporary scene construction is owned by build-state stepping behind a named helper.

## Shipped State

- `cached_edges/build_state/temp_scene.rs` defines
  `paint_root_cached_edge_build_state_temp_scene()` and owns fresh `fret_core::Scene`
  construction.
- `cached_edges/build_state/step.rs` creates temporary scenes internally for edge and edge-label
  budgeted build-state steps, then keeps the existing `finish_build_state_step` merge path.
- `cached_edges/edges/single.rs` no longer allocates or passes a local `tmp` scene.
- `cached_edges/edges/tiled.rs` no longer allocates or passes a local `tmp` scene.
- `cached_edges/labels/single.rs` no longer allocates or passes a local `tmp` scene.
- `cached_edges/labels/tiled.rs` no longer allocates or passes a local `tmp` scene.
- `ecosystem/fret-node/src/lib.rs` has focused source-policy coverage proving route helpers stay off
  direct temporary scene construction.

## Scope Held

The following operation families remain intentionally outside this lane:

- cache-local clip-op construction and merge policy,
- replay scene sinks,
- cache key semantics,
- route-input host/services/scale adapters,
- overlay routing.

## Follow-On Recommendation

Start a separate lane for cache-local clip-op construction and merge cleanup. That lane should focus
on `build_state/ops.rs` and avoid mixing in replay or route-input adapter changes.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/temp_scene.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/step.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/tiled.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/tiled.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/fret-node-paint-root-cached-edge-build-state-temp-scene-adapter-v1/EVIDENCE_AND_GATES.md`

## Gates

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_build_state_temp_scene_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risk

`build_state/ops.rs` still owns cache-local clip stack construction and merge policy. That is
intentional for this lane and should be handled as a separate operation-family follow-on.
