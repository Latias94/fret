# Fret Node Paint Root Cached Edge Replay Adapter v1 - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

This lane is closed.

It follows `fret-node-paint-root-cached-static-scene-adapter-v1` and owns cached edge and edge-label
replay scene sinks only. Edge build-state route inputs, temporary scenes, cache keys, and overlays
are out of scope.

## Shipped Action

Implemented `cached_edges/replay_adapter.rs` and `cached_edges/replay_retained_cx.rs`, updated
`cached_edges/edges/replay.rs` and `cached_edges/labels/replay.rs`, then add source-policy coverage
in `ecosystem/fret-node/src/lib.rs`.

## Validation

Run:

```bash
cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_replay_adapter
```

The full gate set in `EVIDENCE_AND_GATES.md` passed before closeout.

## Residual Follow-ons

- cached edge build-state host/services/scale route-input adapter,
- cached edge-label build-state route-input adapter,
- cached local clip-op emission adapter,
- immediate edge/overlay pass routing adapter.
