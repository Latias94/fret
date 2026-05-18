# WGPU Materials Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`wgpu-conformance-harness-v1` extracted shared WGPU integration-test helpers, and
`wgpu-clip-conformance-harness-followon-v1` proved the same helper can cleanly serve another
renderer test family. The material conformance tests still carry local copies of the same
final-render readback path.

This lane removes that duplication from the material test family while preserving the tests that
serve as evidence for ADR 0235, ADR 0242, and ADR 0274.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: both material tests use `Rgba8Unorm`,
  transparent clear, scale factor `1.0`, and final RGBA sampling, matching
  `tests/support::render_scene_rgba8`. If wrong, the material conformance gate should fail.
- Confident: the slice should stay limited to `materials_conformance.rs` and
  `materials_sampled_conformance.rs`. Evidence: these two files share the same material contract
  family and helper shape. If wrong, paint or effect tests should get their own follow-on.
- Likely: ADR alignment docs do not need content changes because behavior evidence file paths remain
  the same. Evidence: the tests stay in place and only import shared support. If wrong, update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent evidence anchors.
- Likely: preserving explicit `render_scene_rgba8(..., 1.0)` at callsites is clearer than adding a
  convenience wrapper just for 1x material tests. If wrong, a helper can be added after more
  callsites prove that shape.

## Target State

- `materials_conformance.rs` imports `tests/support` for final scene readback and pixel sampling.
- `materials_sampled_conformance.rs` imports `tests/support` for final scene readback and pixel
  sampling.
- Existing material registration, fallback, budget-pressure, sampled catalog texture assertions, and
  adapter skip behavior remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing material semantics or shader behavior.
- Updating ADR status.
- Migrating paint, effect, mask, text, MSAA, or viewport tests.
- Moving test support into production crates.
- Adding new fixture macros or DSLs.

## First Slice

`WMH-010`: migrate the two material-related tests onto `tests/support` and run the affected
materials gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after the two named material-related tests migrated to the shared WGPU test
support module. Future renderer test families should use narrower follow-ons.
