# Fret Node Paint Root Cached Edge Build State Adapter v1 - Closeout Audit

Date: 2026-05-25
Status: Closed

## Closeout Claim

Cached edge build-state host/services/scale route inputs are isolated behind a named adapter seam.
The lane did not include cached edge-label build-state routing.

## Shipped State

- `cached_edges/build_state_adapter.rs` defines `PaintRootCachedEdgeBuildStateCx` and the route-input
  contract for cached edge build-state initialization and budget stepping.
- `cached_edges/build_state_retained_cx.rs` is the retained `PaintCx` binding and owns `app`,
  `services`, and `scale_factor` field reads.
- `cached_edges/edges/single.rs` no longer reads `cx.app`, `cx.services`, or `cx.scale_factor` for
  cached edge build-state routing.
- `cached_edges/edges/tiled.rs` no longer reads `cx.app`, `cx.services`, or `cx.scale_factor` for
  cached edge build-state routing.
- `ecosystem/fret-node/src/lib.rs` has focused source-policy coverage for the adapter, retained
  binding, edge route helpers, and the edge-label scope guard.

## Scope Held

The following operation families remain intentionally outside this lane:

- cached edge-label build-state host/services/scale route inputs,
- cache-local temporary scene construction,
- cache-local clip-op construction,
- overlay routing.

## Follow-On Recommendation

Start a separate lane for cached edge-label build-state route inputs. It should mirror the same
operation-family scope discipline and avoid merging label budget-step work with temporary-scene or
overlay cleanup.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/tiled.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/fret-node-paint-root-cached-edge-build-state-adapter-v1/EVIDENCE_AND_GATES.md`

## Gates

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_build_state_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risk

Cached edge-label build-state routing still reads retained host/services/scale fields directly by
design because this lane kept labels out of scope. That is a known follow-on, not an incomplete
piece of this lane.
