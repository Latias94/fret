# Text Atlas Debug Facade Owner v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is complete. Native-only atlas debug dimensions and lookup facades are owned by
`text/diagnostics_debug.rs`; general diagnostics remain focused on telemetry snapshots.

## Important Invariant

Keep `diagnostics_debug.rs` native-only. The methods it provides are consumed by native renderer
text dumps, while wasm builds compile without exposing these facade methods.

## Future Work

The remaining native-only atlas debug lookup internals still live in `atlas.rs` and
`atlas_runtime_state.rs`. If that surface grows, split an `atlas_debug` owner module rather than
putting more dump-specific facade logic back into general diagnostics.
