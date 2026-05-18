# Renderer Render Plan Postprocess Tests Owner v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is moving postprocess-specific render-plan tests into a matching test owner module. The
intended code change is a test organization split only.

## Important Invariant

Do not change assertions or production behavior in this lane. Shared test helpers may remain in root
`tests.rs` until a broader test-helper owner is justified.

## Future Work

Future slices can split debug validation tests and lifecycle analysis tests once this postprocess
test owner is closed.
