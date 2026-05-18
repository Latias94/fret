# WGPU Test Support Dead Code Prune v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This follow-on is complete. `fret-render-wgpu` no longer has `dead_code` allowances in production or
test code.

## Important Invariant

Integration tests compile independently. If a test only needs raw readback, import
`support/readback.rs`; if it needs explicit output-format scene rendering, import
`support/render_format.rs`; otherwise use the default `support/mod.rs` scene-rendering facade.

## Future Work

A larger fixture-driven conformance harness can still collapse more duplicated test setup, but it
should be a separate lane. This lane intentionally only removed the dead-code allowance without
rewriting the suite.
