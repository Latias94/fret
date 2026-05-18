# WGPU Backdrop Warp V2 Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The BackdropWarpV2 conformance test uses the shared WGPU test support module for
final `Rgba8Unorm` render/readback and pixel sampling.

## Completed

- Removed duplicated local `read_texture_rgba8`, `pixel_rgba`, and `render_and_readback` helpers from
  `crates/fret-render-wgpu/tests/effect_backdrop_warp_v2_conformance.rs`.
- Preserved deterministic warp-map image registration, missing-image fallback behavior, and
  FilterContent ignore checks.
- Recorded gates and closeout evidence.

## Continue Policy

Do not reopen this lane for unrelated conformance families. Start a narrow follow-on for each
remaining family whose helper shape is proven compatible with shared support.

Recommended follow-ons:

- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA tests only after format and
  metadata differences are audited.
