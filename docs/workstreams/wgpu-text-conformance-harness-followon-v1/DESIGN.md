# WGPU Text Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module owns the common `Rgba8Unorm` final-render readback
path used by many renderer conformance tests. The text paint and text outline tests still kept local
copies of that helper shape even though their text-specific behavior lives in deterministic font
setup and assertions, not in the readback mechanics.

This lane removes the duplicated readback helpers while preserving the existing text paint, shadow,
and outline conformance evidence.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: `text_paint_conformance.rs` and
  `text_outline_conformance.rs` render to `Rgba8Unorm`, use transparent clear, sample final RGBA
  pixels, and use scale factor `1.0`, matching `tests/support::render_scene_rgba8`. If wrong, the
  text conformance gate should fail.
- Confident: deterministic font setup must remain local to each test binary. Evidence: both tests
  configure bundled fonts, disable system fonts, and assert fallback policy state before rendering.
  If wrong, text rendering could become host-font dependent.
- Likely: ADR alignment docs do not need content changes because the evidence files named by ADR
  0279 and ADR 0283 remain unchanged. If wrong, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with
  equivalent evidence anchors.
- Likely: image, custom effect, viewport metadata, Vulkan, and MSAA conformance tests should stay in
  separate follow-ons because their setup may differ by asset registration, render target metadata,
  format, or platform assumptions.

## Target State

- `text_paint_conformance.rs` imports `tests/support` for final scene readback and pixel sampling.
- `text_outline_conformance.rs` imports `tests/support` for final scene readback and pixel sampling.
- Existing deterministic font setup, text preparation, scene construction, samples, and assertions
  remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing text paint, text shadow, text outline, glyph rasterization, font fallback, or text cache
  semantics.
- Updating ADR status.
- Migrating image, custom effect, viewport metadata, Vulkan, or MSAA tests.
- Moving test support into production crates.

## First Slice

`WTX-010`: migrate the two text conformance tests onto `tests/support` and run the affected text
conformance gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after the two text conformance tests migrated to the shared WGPU test support
module.
