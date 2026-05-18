# WGPU Backdrop Warp V2 Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`effect_backdrop_warp_v2_conformance.rs` was intentionally left out of the broader backdrop-effects
harness migration because it owns warp-map image registration. After the safer effect families
migrated, this narrow follow-on audits that special setup and removes only the duplicated final
render/readback helpers.

The image registration helper stays local to the test; only the `Rgba8Unorm` final readback and
pixel sampling path moves to shared test support.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence:
  `effect_backdrop_warp_v2_conformance.rs` renders to `Rgba8Unorm`, uses transparent clear, samples
  final RGBA pixels, and uses scale factor `1.0`, matching `tests/support::render_scene_rgba8`. If
  wrong, the BackdropWarpV2 conformance gate should fail.
- Confident: `register_constant_warp_map_rg_signed` must remain test-owned. Evidence: the test
  creates a deterministic 1x1 warp map and registers it as a renderer image before rendering. If
  wrong, the image-driven field coverage would become less explicit.
- Likely: ADR alignment docs do not need content changes because the behavior evidence file remains
  unchanged. If wrong, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent evidence
  anchors.

## Target State

- `effect_backdrop_warp_v2_conformance.rs` imports `tests/support` for final scene readback and
  pixel sampling.
- Existing stripe scene setup, warp-map registration, missing-image fallback checks, FilterContent
  ignore checks, and foreground-order assertions remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing BackdropWarpV2 semantics, image registration, warp-map encoding, sampling hints, or
  missing-image fallback behavior.
- Updating ADR status.
- Migrating image sampling, output transfer, viewport metadata, Vulkan, or MSAA tests.
- Moving test support into production crates.

## First Slice

`WBW2-010`: migrate the BackdropWarpV2 conformance test onto `tests/support` and run the affected
conformance gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `effect_backdrop_warp_v2_conformance.rs` migrated to the shared WGPU test
support module while retaining its local warp-map registration helper.
