# Text Bootstrap Platform Owner v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is complete. `text/bootstrap.rs` owns text-system assembly, while
`text/bootstrap/platform.rs` owns the wasm/native startup `ParleyShaper` policy.

## Important Invariant

Wasm startup remains bundled-only. Native startup continues to follow `ParleyShaper::new()`.

## Future Work

If platform startup grows beyond shaper selection, add a new narrow follow-on rather than expanding
`bootstrap.rs` again.
