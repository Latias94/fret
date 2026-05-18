# WGPU Text Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The named text conformance tests use the shared WGPU test support module for
final `Rgba8Unorm` render/readback and pixel sampling.

## Completed

- Removed duplicated local `read_texture_rgba8`, `pixel_rgba`, and `render_and_readback` helpers from:
  - `crates/fret-render-wgpu/tests/text_paint_conformance.rs`
  - `crates/fret-render-wgpu/tests/text_outline_conformance.rs`
- Preserved deterministic font setup and behavior assertions.
- Recorded gates and closeout evidence.

## Continue Policy

Do not reopen this lane for unrelated conformance families. Start a narrow follow-on for each
remaining family whose helper shape is proven compatible with shared support.

Recommended follow-ons:

- `effect_backdrop_warp_v2_conformance.rs` as an image-registration-aware lane.
- `effect_custom_v1/v2/v3` as a custom effect lane.
- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA tests only after format and
  metadata differences are audited.
