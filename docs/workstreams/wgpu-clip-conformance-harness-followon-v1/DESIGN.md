# WGPU Clip Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`wgpu-conformance-harness-v1` extracted shared integration-test helpers for WGPU scene rendering,
texture readback, and RGBA pixel sampling. That first lane deliberately migrated only the path
conformance batch and closed with a follow-on policy: move another test family only when a concrete
duplication pattern is proven.

The clip-related tests now have that proof. Both `clip_path_conformance.rs` and
`affine_clip_conformance.rs` still duplicate the same final-render readback helpers.

## Assumptions First

- Confident: this is a test-surface refactor, not a renderer semantics change. Evidence:
  `tests/support/mod.rs` already renders with the same `Rgba8Unorm` target, transparent clear,
  caller-provided scene, viewport size, and scale factor. If wrong, the clip gate should fail and
  the behavior fix should split into a semantic renderer lane.
- Confident: the slice should stay limited to `clip_path_conformance.rs` and
  `affine_clip_conformance.rs`. Evidence: they share the same final-render readback shape, while
  other WGPU tests may need different setup policies. If wrong, a future family should open its own
  follow-on.
- Likely: `affine_clip_conformance.rs` still needs its local direct `renderer.render_scene(...)`
  setup for the viewport-source texture. Evidence: that block renders into a texture used later as a
  registered render target, not the final readback target. If wrong, a richer source-texture helper
  can be added after another callsite proves the shape.
- Likely: keeping `render_scene_rgba8(..., 1.0)` at callsites is clearer than adding a convenience
  wrapper for this small follow-on. If wrong, the support API can grow after more tests prove 1x is
  universal.

## Target State

- `clip_path_conformance.rs` imports `tests/support` for final scene readback and pixel sampling.
- `affine_clip_conformance.rs` imports `tests/support` for final scene readback and pixel sampling.
- Existing clip assertions, adapter skip behavior, texture formats, clear behavior, viewport sizes,
  and scale factors remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Migrating every WGPU conformance test.
- Adding new clip behavior coverage.
- Introducing macros or fixture DSLs.
- Moving test support into production crates.
- Changing renderer internals, clip semantics, or viewport-surface behavior.

## First Slice

`WCF-010`: migrate the two clip-related tests onto `tests/support` and run the affected clip gate.

## Closure Policy

Close this lane once the code migration and gates pass. Do not reopen the older path harness lane.

## Closure

Closed on 2026-05-18 after the two named clip-related tests migrated to the shared WGPU test
support module. Future renderer test families should use narrower follow-ons.
