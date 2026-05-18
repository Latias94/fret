# WGPU Path MSAA Composite Vulkan Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`path_msaa_composite_vulkan.rs` still duplicated the shared raw readback helper even though its unique
contract is Vulkan-specific path-MSAA composite visibility across multiple passes.

This lane removes the duplicated raw readback mechanics and routes byte sampling through the shared
pixel helper under a local `pixel_bgra` alias because the test renders to `Bgra8UnormSrgb` and
intentionally reasons about BGRA channel order.

## Assumptions First

- Confident: this is a helper-migration refactor only. Evidence: the local `read_texture_rgba8` has
  the same row-padding/map/readback shape as `tests/support::read_texture_rgba8`. If wrong, the
  Vulkan composite smoke gate should fail.
- Confident: BGRA naming must remain explicit at the call site. Evidence: the output format is
  `Bgra8UnormSrgb` and the assertions explicitly check red/green channels in BGRA order.
- Likely: no ADR update is needed because the behavior evidence file remains the same and only
  delegates raw readback mechanics to shared test support.

## Target State

- `path_msaa_composite_vulkan.rs` imports `tests/support::{read_texture_rgba8, pixel_rgba}` and
  aliases `pixel_rgba` as `pixel_bgra`.
- Vulkan backend guard, MSAA sample setup, explicit BGRA target, and assertions remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing Vulkan path-MSAA behavior, composite pass ordering, output format, or BGRA assertions.
- Replacing this test with `render_scene_rgba8` or RGBA pixel sampling.
- Migrating `vulkan_path_msaa_visibility_conformance.rs`.

## First Slice

`WPMCV-010`: migrate duplicated raw readback helper onto `tests/support` and run the affected Vulkan
composite smoke gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `path_msaa_composite_vulkan.rs` adopted shared raw readback and byte
sampling while retaining BGRA naming and Vulkan/MSAA semantics.
