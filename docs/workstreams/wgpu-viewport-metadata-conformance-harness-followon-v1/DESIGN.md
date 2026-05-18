# WGPU Viewport Metadata Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`viewport_surface_metadata_conformance.rs` still duplicated the shared WGPU final-render readback and
pixel helper shape even though its final output path is the standard `Rgba8Unorm` transparent-clear
path used by `tests/support::render_scene_rgba8`.

This lane removes the duplicated final-render/readback mechanics while preserving the metadata-owned
source textures, render target registration, alpha-mode assertions, and orientation assertions.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: the removed `render_and_readback` helper
  rendered to `Rgba8Unorm`, used transparent clear, used scale factor `1.0`, and sampled RGBA pixels,
  matching `tests/support::render_scene_rgba8`. If wrong, the viewport metadata conformance gate
  should fail.
- Confident: source texture writing and `RenderTargetMetadata` registration remain local to the test.
  Evidence: those steps encode the alpha/orientation contract and are not generic harness mechanics.
- Likely: ADR alignment docs do not need content changes because the behavior evidence file remains
  the same and only delegates final readback mechanics to shared test support. If wrong, refresh
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` anchors for ADR 0234/0282.

## Target State

- `viewport_surface_metadata_conformance.rs` imports `tests/support::{render_scene_rgba8, pixel_rgba}`.
- Local source texture writers, metadata setup, render target registration, and assertions remain
  unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing `RenderTargetMetadata`, alpha handling, orientation mapping, or imported target semantics.
- Updating ADR status.
- Migrating Vulkan, MSAA, or host-topology smoke tests.

## First Slice

`WVM-010`: migrate duplicated final-render/readback helpers onto `tests/support` and run the affected
viewport metadata gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `viewport_surface_metadata_conformance.rs` adopted shared final
render/readback and pixel helpers while retaining metadata setup and assertions locally.
