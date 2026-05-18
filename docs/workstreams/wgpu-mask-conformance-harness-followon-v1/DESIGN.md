# WGPU Mask Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The shared WGPU integration-test support module has already replaced duplicated final-render
readback helpers in path, clip, and material conformance tests. The mask tests still keep local
copies of the same texture creation, readback, and pixel sampling code.

This lane removes that duplication from the mask test family while preserving the tests that serve
as evidence for ADR 0239 and ADR 0273.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: both mask tests use `Rgba8Unorm`,
  transparent clear, scale factor `1.0`, and final RGBA sampling, matching
  `tests/support::render_scene_rgba8`. If wrong, the mask conformance gate should fail.
- Confident: the slice should stay limited to `mask_gradient_conformance.rs` and
  `mask_image_conformance.rs`. Evidence: these two files share the same mask contract family and
  helper shape. If wrong, image sampling or effect tests should get their own follow-on.
- Likely: ADR alignment docs do not need content changes because behavior evidence file paths remain
  the same. Evidence: the tests stay in place and only import shared support. If wrong, update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent evidence anchors.
- Likely: `mask_image_conformance.rs` should keep its alpha-mask upload helper local. Evidence: that
  helper is test setup specific to image masks, not the generic final readback path. If wrong, split
  a later source-texture/test-fixture helper after another callsite proves the shape.

## Target State

- `mask_gradient_conformance.rs` imports `tests/support` for final scene readback and pixel sampling.
- `mask_image_conformance.rs` imports `tests/support` for final scene readback and pixel sampling.
- Existing gradient-mask, image-mask, source switching, deterministic nested degradation assertions,
  alpha-mask upload setup, and adapter skip behavior remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing mask semantics or shader behavior.
- Updating ADR status.
- Migrating paint, effect, text, image sampling, MSAA, or viewport tests.
- Moving test support into production crates.
- Adding new fixture macros or DSLs.

## First Slice

`WMK-010`: migrate the two mask-related tests onto `tests/support` and run the affected mask gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after the two named mask-related tests migrated to the shared WGPU test support
module. Future renderer test families should use narrower follow-ons.
