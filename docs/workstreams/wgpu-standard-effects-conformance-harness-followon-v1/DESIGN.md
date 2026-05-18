# WGPU Standard Effects Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module owns final-render readback and RGBA pixel sampling
for standard `Rgba8Unorm` conformance tests. A set of standard effect and postprocess tests still
carried local copies of the same helper shape.

This lane removes that duplication while preserving the existing render target format, transparent
clear, scale factor `1.0`, scene construction, and pixel assertions.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: the named tests all render to
  `Rgba8Unorm`, use transparent clear, use scale factor `1.0`, and sample final RGBA pixels. If
  wrong, the grouped conformance gate should fail.
- Confident: the explicit all-zero clear in `postprocess_scissor_conformance.rs` is equivalent to
  `wgpu::Color::TRANSPARENT`. If wrong, the outside-region alpha assertions should fail.
- Likely: backdrop, custom, warp-v2, drop-shadow, image, text, viewport metadata, Vulkan, and MSAA
  tests should not be included here. Evidence: several of those tests own additional descriptors,
  image setup, float textures, metadata assertions, or platform-specific target formats. If wrong,
  migrate each family in a separate follow-on after auditing its setup.
- Likely: ADR alignment docs do not need content updates because behavior evidence file paths stay
  the same and only harness plumbing changed. If wrong, update the ADR implementation matrix with
  equivalent evidence anchors.

## Target State

- The named standard effect/postprocess tests import `support::{pixel_rgba, render_scene_rgba8}`.
- Local copies of `read_texture_rgba8`, `pixel_rgba`, and `render_and_readback` are deleted from
  those files.
- Existing effect chains, render budget setup, and assertions remain unchanged.
- The lane closes after the narrow migration and gates pass.

## Out Of Scope

- Changing effect, postprocess, blend, clip, or filter behavior.
- Migrating backdrop, custom, warp-v2, drop-shadow, image, text, viewport metadata, Vulkan, MSAA, or
  paint-eval-space tests.
- Moving integration-test support into production crates.
- Updating public renderer APIs.

## First Slice

`WSE-010`: migrate the named standard effect/postprocess tests onto shared support and run the
grouped conformance gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after the named standard effect/postprocess tests migrated to the shared WGPU
test support module while preserving `Rgba8Unorm` transparent-clear behavior.
