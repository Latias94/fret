# Renderer Render Plan Debug Validation Tests Owner v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is moving debug-validation-specific render-plan tests into a matching test owner module.
The intended code change is a test organization split only.

## Important Invariant

Do not change assertions or validator behavior in this lane. Shared test helpers may remain in root
`tests.rs` until a broader test-helper owner is justified.

## Future Work

Future slices can split lifecycle analysis tests and compiler/effects guardrail tests once this
debug validation test owner is closed.
