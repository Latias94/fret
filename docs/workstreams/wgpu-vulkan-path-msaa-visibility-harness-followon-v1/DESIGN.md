# WGPU Vulkan Path MSAA Visibility Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`vulkan_path_msaa_visibility_conformance.rs` was the last WGPU conformance test duplicating the
shared RGBA8 readback and pixel helper functions.

This lane removes only the duplicated helper mechanics while preserving the Vulkan backend guard,
capability checks, env opt-out guard, path-MSAA perf assertions, explicit BGRA output target, and
visibility assertions.

## Assumptions First

- Confident: this is a helper-migration refactor only. Evidence: the local helpers match
  `tests/support::{read_texture_rgba8, pixel_rgba}` in row-padding, map/readback, and byte sampling.
  If wrong, the Vulkan MSAA visibility gate should fail.
- Confident: the test's Vulkan/MSAA safety-valve contract remains local. Evidence: env guard,
  `TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE`, perf snapshot checks, and degradation assertions
  are unchanged.
- Likely: no ADR update is needed because behavior evidence remains in the same file and only test
  harness duplication is removed.

## Target State

- `vulkan_path_msaa_visibility_conformance.rs` imports
  `tests/support::{read_texture_rgba8, pixel_rgba}`.
- Env locking, Vulkan capability checks, path-MSAA setup, perf assertions, and alpha visibility
  checks remain unchanged.
- The lane closes after this final helper migration.

## Out Of Scope

- Changing Vulkan path-MSAA defaults, env opt-out behavior, perf snapshot semantics, or output
  format.
- Moving env guard logic into shared support.
- Adding a BGRA-specific helper because the current assertions only sample alpha.

## First Slice

`WVMV-010`: migrate duplicated readback/pixel helpers onto `tests/support` and run the affected
Vulkan MSAA visibility gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `vulkan_path_msaa_visibility_conformance.rs` adopted shared readback and
pixel helpers while retaining Vulkan/MSAA visibility semantics.
