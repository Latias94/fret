# Fret Node Paint Root Cached Static Scene Adapter v1 - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

This lane is closed.

It follows `fret-node-paint-root-pass-clip-adapter-v1` and owns cached static group/node replay
only. Cached edge replay, edge labels, overlays, and cache key semantics are out of scope.

## Shipped Action

Implemented `cached_static_scene_adapter.rs` and `cached_static_scene_retained_cx.rs`, updated
`static_layer.rs`, `static_cache.rs`, `cached_groups.rs`, and `cached_nodes.rs`, then add
source-policy coverage in `ecosystem/fret-node/src/lib.rs`.

## Validation

Run:

```bash
cargo test -p fret-node --features compat-retained-canvas paint_root_cached_static_scene_adapter
```

The full gate set in `EVIDENCE_AND_GATES.md` passed before closeout.

## Residual Follow-ons

- cached edge scene replay adapter,
- cached edge-label scene replay adapter,
- cached local clip-op emission adapter,
- immediate edge/overlay pass routing adapter.
