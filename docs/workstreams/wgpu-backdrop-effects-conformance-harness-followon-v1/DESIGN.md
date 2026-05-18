# WGPU Backdrop Effects Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module owns final-render readback and RGBA pixel sampling
for standard `Rgba8Unorm` conformance tests. The backdrop effect tests still carried local copies of
the same helper shape.

This lane removes that duplication while preserving the existing render target format, transparent
clear, scale factor `1.0`, scene construction helpers, and pixel assertions.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: the named tests render to
  `Rgba8Unorm`, use transparent clear, use scale factor `1.0`, and sample final RGBA pixels. If
  wrong, the grouped conformance gate should fail.
- Confident: local non-readback helpers must remain local. Evidence: `is_grayish`,
  `push_bounds_stripes`, and `stripe_scene_base` describe test scenarios/assertions, not texture
  readback plumbing. If wrong, compile or behavior gates should fail.
- Likely: `effect_backdrop_warp_v2_conformance.rs` should remain out of scope. Evidence: it owns
  image registration setup and should be audited separately from plain scene readback. If wrong,
  migrate it in a dedicated follow-on after checking its image setup.
- Likely: ADR alignment docs do not need content updates because behavior evidence file paths stay
  the same and only harness plumbing changed. If wrong, update the ADR implementation matrix with
  equivalent evidence anchors.

## Target State

- The named backdrop effect tests import `support::{pixel_rgba, render_scene_rgba8}`.
- Local copies of `read_texture_rgba8`, `pixel_rgba`, and `render_and_readback` are deleted from
  those files.
- Existing backdrop effect chains, render budget setup, helper scene builders, and assertions remain
  unchanged.
- The lane closes after the narrow migration and gates pass.

## Out Of Scope

- Changing backdrop blur, acrylic, color-adjust, pixelate, warp, clip, or ordering behavior.
- Migrating `effect_backdrop_warp_v2_conformance.rs`.
- Migrating custom effects, drop shadow, image, text, viewport metadata, Vulkan, MSAA, or
  paint-eval-space tests.
- Moving integration-test support into production crates.
- Updating public renderer APIs.

## First Slice

`WBE-010`: migrate the named backdrop effect tests onto shared support and run the grouped
conformance gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after the named backdrop effect tests migrated to the shared WGPU test support
module while preserving `Rgba8Unorm` transparent-clear behavior.
