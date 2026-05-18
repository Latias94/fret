# WGPU Custom Effects Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The CustomV1/V2/V3 conformance tests use the shared WGPU test support module for
final `Rgba8Unorm` render/readback and pixel sampling.

## Completed

- Removed duplicated local `read_texture_rgba8`, `pixel_rgba`, and `render_and_readback` helpers from:
  - `crates/fret-render-wgpu/tests/effect_custom_v1_conformance.rs`
  - `crates/fret-render-wgpu/tests/effect_custom_v2_conformance.rs`
  - `crates/fret-render-wgpu/tests/effect_custom_v3_conformance.rs`
- Preserved custom-effect WGSL registration, user-image registration, perf counters, budget
  degradation checks, and behavior assertions.
- Recorded gates and closeout evidence.

## Continue Policy

Do not reopen this lane for unrelated conformance families. Start a narrow follow-on for each
remaining family whose helper shape is proven compatible with shared support.

Recommended follow-ons:

- `effect_backdrop_warp_v2_conformance.rs` as an image-registration-aware lane.
- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA tests only after format and
  metadata differences are audited.
