# WGPU Stroke Dash Shadow Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module owns final-render readback and RGBA pixel sampling
for `Rgba8Unorm` conformance tests. The stroke, dash, and rounded-shadow tests still carried local
copies of the same helper shape.

This lane removes that duplication while preserving the existing render target format, transparent
clear, scale-factor loops, scene construction, and pixel assertions.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence:
  `dashed_border_conformance.rs`, `dash_semantics_rrect_vs_path_conformance.rs`,
  `stroke_paint_conformance.rs`, and `shadow_rrect_conformance.rs` all render to `Rgba8Unorm`, use
  transparent clear, pass a caller-provided `scale_factor`, and sample final RGBA pixels. If wrong,
  the grouped conformance gate should fail.
- Confident: the local clear-color spellings are equivalent to `wgpu::Color::TRANSPARENT`.
  Evidence: the explicit clears use all-zero RGBA values. If wrong, transparent-background alpha
  assertions should fail.
- Likely: keeping the shared helper in integration-test support is the correct boundary. Evidence:
  the helper depends on `fret_render_wgpu::{Renderer, WgpuContext}` and should not become a
  production renderer API. If wrong, a broader test-support crate can be split later.
- Likely: ADR alignment docs do not need content updates because behavior evidence file paths stay
  the same and only harness plumbing changed. If wrong, update the ADR implementation matrix with
  equivalent evidence anchors.

## Target State

- The named stroke, dash, and shadow tests import `support::{pixel_rgba, render_scene_rgba8}`.
- Local copies of `read_texture_rgba8`, `pixel_rgba`, and `render_and_readback` are deleted from
  those files.
- Existing scale-factor loops and assertions remain unchanged.
- The lane closes after the narrow migration and gates pass.

## Out Of Scope

- Changing stroke, dash, gradient, or shadow rendering behavior.
- Migrating effect, text, image, viewport, Vulkan, MSAA, or paint-eval-space tests.
- Moving integration-test support into production crates.
- Updating public renderer APIs.

## First Slice

`WSDS-010`: migrate the four named tests onto shared support and run the grouped conformance gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after the named stroke, dash, and shadow tests migrated to the shared WGPU
test support module while preserving `Rgba8Unorm` transparent-clear behavior and scale-factor
coverage.
