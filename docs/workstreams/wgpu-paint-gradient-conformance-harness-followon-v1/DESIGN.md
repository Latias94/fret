# WGPU Paint Gradient Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module has already replaced duplicated final-render
readback helpers across several renderer test families. `paint_gradient_conformance.rs` still keeps
the same local `Rgba8Unorm` final render/readback/pixel helper shape.

This lane removes that duplication while preserving the gradient tests that serve as evidence for
ADR 0233, ADR 0274, and ADR 0280.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: `paint_gradient_conformance.rs` uses
  `Rgba8Unorm`, transparent clear, scale factor `1.0`, and final RGBA sampling, matching
  `tests/support::render_scene_rgba8`. If wrong, the paint-gradient conformance gate should fail.
- Confident: this slice should not include `composite_group_conformance.rs`. Evidence: composite
  conformance renders to `Rgba8UnormSrgb`, while the current shared helper is intentionally
  `Rgba8Unorm`-only. If wrong, a separate helper-format follow-on should first add a format-aware
  support API.
- Likely: ADR alignment docs do not need content changes because behavior evidence file paths remain
  the same. Evidence: the test stays in place and only imports shared support. If wrong, update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent evidence anchors.
- Likely: keeping `render_scene_rgba8(..., 1.0)` explicit is clearer than adding a 1x wrapper for
  this one-file lane.

## Target State

- `paint_gradient_conformance.rs` imports `tests/support` for final scene readback and pixel
  sampling.
- Existing linear/radial/sweep gradient assertions, Oklab midpoint check, tile-mode checks, and
  adapter skip behavior remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing paint, gradient, color-space, or shader semantics.
- Updating ADR status.
- Migrating composite, effect, text, image sampling, MSAA, or viewport tests.
- Adding a format-aware support helper.
- Moving test support into production crates.

## First Slice

`WPG-010`: migrate `paint_gradient_conformance.rs` onto `tests/support` and run the affected
paint-gradient gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `paint_gradient_conformance.rs` migrated to the shared WGPU test support
module. Composite-group conformance remains intentionally out of scope until a format-aware helper is
introduced in a separate lane.
