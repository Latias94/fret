# WGPU Paint Eval Space Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module now owns the common `Rgba8Unorm` final-render
readback path used by many renderer conformance tests. The paint evaluation-space tests still kept
local copies of the same readback, pixel sampling, and render-to-texture helpers.

This lane removes that duplication while preserving the paint evaluation-space evidence for
viewport-space gradients and stroke-S01 path/rrect gradients.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence:
  `paint_eval_space_stroke_s01_conformance.rs` and `paint_eval_space_viewport_conformance.rs` render
  to `Rgba8Unorm`, use transparent clear, sample final RGBA pixels, and already pass explicit
  scale factors through their local helpers. If wrong, the paint evaluation-space conformance gate
  should fail.
- Confident: preserving each test's `scale_factor` argument is part of the contract. Evidence: both
  tests iterate over `1.0`, `1.5`, and `2.0`, and `tests/support::render_scene_rgba8` accepts the
  same scale-factor parameter.
- Likely: ADR alignment docs do not need content changes because the behavior evidence files remain
  in place and only delegate readback mechanics to shared test support. If wrong, update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent evidence anchors.
- Likely: remaining image, text, custom effect, viewport metadata, Vulkan, and MSAA conformance
  tests should stay in separate follow-ons because their setup may differ by asset registration,
  render target metadata, format, or platform assumptions.

## Target State

- `paint_eval_space_stroke_s01_conformance.rs` imports `tests/support` for final scene readback and
  pixel sampling.
- `paint_eval_space_viewport_conformance.rs` imports `tests/support` for final scene readback and
  pixel sampling.
- Existing scene construction, path preparation, scale-factor loops, samples, and assertions remain
  unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing paint, gradient, stroke parameterization, shader, or scale-factor semantics.
- Updating ADR status.
- Migrating text, image, custom effect, viewport metadata, Vulkan, MSAA, or warp-v2 tests.
- Moving test support into production crates.

## First Slice

`WPE-010`: migrate the two paint evaluation-space conformance tests onto `tests/support` and run the
affected paint evaluation-space gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after the two paint evaluation-space conformance tests migrated to the shared
WGPU test support module.
