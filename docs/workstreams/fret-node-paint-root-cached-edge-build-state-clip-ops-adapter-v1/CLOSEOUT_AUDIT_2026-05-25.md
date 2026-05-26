# Fret Node Paint Root Cached Edge Build State Clip Ops Adapter v1 - Closeout Audit

Date: 2026-05-25
Status: Closed

## Closeout Claim

Cached edge build-state cache-local clip stack construction and temp-op merge policy are isolated
behind a named helper. `build_state/ops.rs` no longer owns direct `PushClipRect`/`PopClip` policy.

## Shipped State

- `cached_edges/build_state/clip_ops.rs` defines
  `paint_root_cached_edge_build_state_initial_clip_ops()` and
  `paint_root_cached_edge_build_state_merge_temp_ops()`.
- `clip_ops.rs` owns `SceneOp::PushClipRect`, `SceneOp::PopClip`, and the trailing pop-clip merge
  behavior.
- `cached_edges/build_state/ops.rs` keeps `finish_build_state_step` completion bookkeeping and
  delegates clip construction/merge policy.
- `ecosystem/fret-node/src/lib.rs` has focused source-policy coverage proving the helper boundary.

## Scope Held

The following operation families remain intentionally outside this lane:

- temporary scene construction,
- replay scene sinks,
- cache key semantics,
- route-input host/services/scale adapters,
- overlay routing.

## Follow-On Recommendation

Prefer a paint-root overlay routing or replay/cache-key cleanup lane next. The cached edge
build-state internals now have separate seams for route inputs, temporary scenes, and clip ops.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/clip_ops.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/ops.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/fret-node-paint-root-cached-edge-build-state-clip-ops-adapter-v1/EVIDENCE_AND_GATES.md`

## Gates

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_build_state_clip_ops_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risk

No behavior change was intended. The helper preserves the existing trailing `PopClip` sentinel
behavior; broader cached edge overlay/replay/cache-key cleanup remains separate.
