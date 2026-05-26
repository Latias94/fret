# Fret Node Paint Root Cached Edge Label Build State Adapter v1 - Closeout Audit

Date: 2026-05-25
Status: Closed

## Closeout Claim

Cached edge-label build-state host/services/scale route inputs are isolated behind a named adapter
seam. The lane did not reopen cached edge build-state routing.

## Shipped State

- `cached_edges/label_build_state_adapter.rs` defines `PaintRootCachedEdgeLabelBuildStateCx` and the
  route-input contract for cached edge-label build-state initialization and budget stepping.
- `cached_edges/label_build_state_retained_cx.rs` is the retained `PaintCx` binding and owns `app`,
  `services`, and `scale_factor` field reads.
- `cached_edges/labels/single.rs` no longer reads `cx.app`, `cx.services`, or `cx.scale_factor` for
  cached edge-label build-state routing.
- `cached_edges/labels/tiled.rs` no longer reads `cx.app`, `cx.services`, or `cx.scale_factor` for
  cached edge-label build-state routing.
- `ecosystem/fret-node/src/lib.rs` has focused source-policy coverage for the adapter, retained
  binding, label route helpers, and the edge build-state scope guard.

## Scope Held

The following operation families remain intentionally outside this lane:

- cached edge build-state routing,
- cached edge and edge-label replay scene sinks,
- cache-local temporary scene construction,
- cache-local clip-op construction,
- overlay routing.

## Follow-On Recommendation

Choose the next slice by operation family. The cleanest next candidate is cache-local temporary scene
construction in cached edge/label build-state stepping, because both build-state lanes still allocate
and drain `fret_core::Scene` directly inside the route helpers.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/label_build_state_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/label_build_state_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/tiled.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/fret-node-paint-root-cached-edge-label-build-state-adapter-v1/EVIDENCE_AND_GATES.md`

## Gates

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_label_build_state_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risk

Cached edge and edge-label build-state route helpers still construct cache-local temporary scenes and
append cache-local clip ops directly. That is the next internal cleanup surface, not an incomplete
piece of this lane.
