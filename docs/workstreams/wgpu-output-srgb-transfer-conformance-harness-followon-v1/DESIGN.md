# WGPU Output sRGB Transfer Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`output_srgb_transfer_conformance.rs` still duplicates the shared RGBA8 readback and pixel helper
functions used by other WGPU conformance tests.

This lane removes only the duplicated readback mechanics while preserving the test-owned explicit
output format, render-target setup, and sRGB transfer assertions.

## Assumptions First

- Confident: this is a helper-migration refactor only. Evidence: the file currently defines local
  `read_texture_rgba8` and `pixel_rgba`, while the actual sRGB transfer expectation lives in the
  test body. If wrong, the conformance gate will expose behavior drift.
- Confident: the test should keep open-coded output texture creation. Evidence: the format under test
  is part of the assertion surface, so a generic render-target helper would blur the contract.
- Likely: no ADR update is needed if the helper migration stays behavior-preserving. If wrong, update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with the same evidence anchors.

## Target State

- `output_srgb_transfer_conformance.rs` imports `tests/support::{read_texture_rgba8, pixel_rgba}`.
- The explicit output texture setup and sRGB transfer assertions remain unchanged.
- The lane closes once the helper duplication is removed and the targeted gate passes.

## Out Of Scope

- Changing output encoding semantics, quantization, or transfer math.
- Generalizing a new render-target helper abstraction.
- Migrating viewport metadata, Vulkan, MSAA, or host-topology smoke tests.

## First Slice

`WOS-010`: migrate duplicated output-sRGB readback/pixel helpers onto `tests/support` and run the
affected conformance gate.

## Closure Policy

Close this lane once the code migration and gate pass.

## Closure

Closed on 2026-05-18 after `output_srgb_transfer_conformance.rs` adopted shared readback/pixel
helpers while retaining explicit output texture setup and sRGB transfer assertions.
