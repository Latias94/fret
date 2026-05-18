# WGPU Composite Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module now owns final-render readback and RGBA pixel
sampling for the `Rgba8Unorm` conformance tests. `composite_group_conformance.rs` still carried a
local copy because its assertions intentionally render to `Rgba8UnormSrgb`.

This lane makes the output format explicit in shared test support, then migrates the composite
test without changing composite, blend, opacity, scissor, or intermediate-budget behavior.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence:
  `composite_group_conformance.rs` uses a transparent clear, scale factor `1.0`, final RGBA
  sampling, and `Rgba8UnormSrgb`; the new shared helper exposes exactly those knobs. If wrong, the
  composite conformance gate should fail.
- Confident: preserving `Rgba8UnormSrgb` is required. Evidence: the blend-mode smoke test computes
  expected values through `linear_to_srgb_f32`. If wrong, switching to `Rgba8Unorm` would silently
  test a different color-space outcome.
- Likely: existing `render_scene_rgba8` callers should remain unchanged. Evidence: they already
  depend on `Rgba8Unorm`; the format-aware helper is additive and the default wrapper keeps the old
  call sites stable. If wrong, the backend test compile gate should expose signature drift.
- Likely: ADR alignment docs do not need a content update because the behavior evidence file path
  remains the same and only the test harness plumbing changed. If wrong, update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent evidence anchors.

## Target State

- `crates/fret-render-wgpu/tests/support/mod.rs` exposes a format-aware final-render helper and
  keeps `render_scene_rgba8` as the `Rgba8Unorm` convenience wrapper.
- `composite_group_conformance.rs` imports shared support for final scene readback and pixel
  sampling.
- The composite test keeps an explicit local wrapper naming its `Rgba8UnormSrgb` contract.
- The lane closes after the narrow migration and gates pass.

## Out Of Scope

- Changing composite, blend, opacity, scissor, or intermediate texture behavior.
- Migrating effect, text, stroke, viewport, MSAA, or remaining paint-eval tests.
- Moving integration-test support into production crates.
- Updating public renderer APIs.

## First Slice

`WCG-010`: add a format-aware shared render/readback helper and migrate
`composite_group_conformance.rs` onto it.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `composite_group_conformance.rs` migrated to the shared WGPU test
support module while preserving `Rgba8UnormSrgb` output semantics.
