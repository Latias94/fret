# WGPU Custom Effects Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module owns the common `Rgba8Unorm` final-render readback
path used by many renderer conformance tests. The CustomV1, CustomV2, and CustomV3 conformance tests
still kept local copies of the same helper shape.

This lane removes that duplication while preserving the custom-effect ABI evidence. Image
registration, custom-effect registration, perf counters, budget assertions, and source/pyramid
coverage stay local to the tests.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: all three custom-effect conformance
  tests render to `Rgba8Unorm`, use transparent clear, sample final RGBA pixels, and use scale
  factor `1.0`, matching `tests/support::render_scene_rgba8`. If wrong, the custom-effect
  conformance gate should fail.
- Confident: image registration and custom-effect registration helpers remain test-owned. Evidence:
  V2 and V3 create/register user textures, non-filterable fallback textures, and custom WGSL effects
  before rendering. If wrong, the ABI coverage would become less explicit.
- Likely: ADR/alignment docs do not need content changes because the evidence files remain
  unchanged. If wrong, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent evidence
  anchors.

## Target State

- `effect_custom_v1_conformance.rs` imports `tests/support` for final scene readback and pixel
  sampling.
- `effect_custom_v2_conformance.rs` imports `tests/support` for final scene readback and pixel
  sampling.
- `effect_custom_v3_conformance.rs` imports `tests/support` for final scene readback and pixel
  sampling.
- Existing effect registration, image registration, perf snapshot checks, budget degradation checks,
  and assertions remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing custom-effect ABI semantics, WGSL stitching, registration, image input compatibility,
  source/pyramid behavior, or budget/degradation policy.
- Updating ADR status.
- Migrating image, output transfer, viewport metadata, Vulkan, or MSAA tests.
- Moving test support into production crates.

## First Slice

`WCE-010`: migrate the three custom-effect conformance tests onto `tests/support` and run the
affected custom-effect conformance gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after the CustomV1/V2/V3 conformance tests migrated to the shared WGPU test
support module.
