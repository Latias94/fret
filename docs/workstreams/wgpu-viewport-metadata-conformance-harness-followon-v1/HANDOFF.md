# WGPU Viewport Metadata Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The viewport metadata conformance test uses shared WGPU test support for final
`Rgba8Unorm` render/readback and pixel sampling while keeping source texture setup and
`RenderTargetMetadata` assertions local.

## Completed

- Removed duplicated local `read_texture_rgba8`, `pixel_rgba`, and `render_and_readback` helpers from
  `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`.
- Preserved source texture writers, alpha-mode metadata assertions, orientation metadata assertions,
  and render target update coverage.
- Recorded gates and closeout evidence.

## Continue Policy

Do not reopen this lane for unrelated backend/platform helper cleanup. Start separate narrow
follow-ons for Vulkan, MSAA, or host-topology tests.
