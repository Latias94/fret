# WGPU Drop Shadow Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module owns the common `Rgba8Unorm` final-render readback
path used by many renderer conformance tests. `effect_drop_shadow_v1_conformance.rs` still kept a
local copy of the same helper shape.

This lane removes that duplication while preserving the drop-shadow evidence for ADR 0286.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence:
  `effect_drop_shadow_v1_conformance.rs` renders to `Rgba8Unorm`, uses transparent clear, samples
  final RGBA pixels, and uses scale factor `1.0`, matching `tests/support::render_scene_rgba8`. If
  wrong, the drop-shadow conformance gate should fail.
- Confident: this should not reopen `wgpu-stroke-dash-shadow-conformance-harness-followon-v1`.
  Evidence: that lane is closed and covered stroke/dash/path shadow families, while this slice owns
  the late standalone `DropShadowV1` effect test.
- Likely: ADR alignment docs do not need content changes because the evidence file named by ADR
  0286 remains unchanged. If wrong, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent
  evidence anchors.

## Target State

- `effect_drop_shadow_v1_conformance.rs` imports `tests/support` for final scene readback and pixel
  sampling.
- Existing DropShadowV1 setup, intermediate budget, scissoring checks, and compositing assertions
  remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing DropShadowV1 semantics, blur pipelines, intermediate budget behavior, or effect
  compilation.
- Updating ADR status.
- Migrating custom effects, image, viewport metadata, Vulkan, or MSAA tests.
- Moving test support into production crates.

## First Slice

`WDS-010`: migrate `effect_drop_shadow_v1_conformance.rs` onto `tests/support` and run the affected
drop-shadow conformance gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `effect_drop_shadow_v1_conformance.rs` migrated to the shared WGPU test
support module.
