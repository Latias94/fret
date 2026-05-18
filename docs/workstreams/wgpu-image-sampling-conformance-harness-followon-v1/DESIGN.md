# WGPU Image Sampling Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`image_sampling_hint_conformance.rs` still duplicated the common RGBA8 readback and pixel helper
functions. Unlike most migrated tests, it also owns explicit render-target setup in each test case
because the assertions are tied to image sampling behavior and output labels.

This lane removes the duplicated readback helpers while preserving the test-owned render target and
checkerboard image registration setup.

## Assumptions First

- Confident: this is a test-surface refactor only. Evidence: the duplicated
  `read_texture_rgba8`/`pixel_rgba` helpers are byte-for-byte equivalent in behavior to
  `tests/support`, while the two test bodies keep their explicit render target creation and
  `render_scene` calls. If wrong, the image sampling conformance gate should fail.
- Confident: this lane should not force `render_scene_rgba8` adoption. Evidence: this test uses
  per-case texture labels and intentionally open-coded render-target setup. If wrong, a separate
  helper API can be introduced after more image tests are audited.
- Likely: ADR alignment docs do not need content changes because the behavior evidence file remains
  unchanged. If wrong, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with equivalent evidence
  anchors.

## Target State

- `image_sampling_hint_conformance.rs` imports `tests/support::{read_texture_rgba8, pixel_rgba}`.
- Existing checkerboard registration, explicit render target creation, image sampling assertions,
  and mixed primitive ordering checks remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing image sampling semantics, sampler selection, bind groups, UI plumbing, or ADR status.
- Introducing a new render-target helper abstraction.
- Migrating output transfer, viewport metadata, Vulkan, MSAA, or host-topology smoke tests.

## First Slice

`WIS-010`: migrate duplicated image-sampling readback/pixel helpers onto `tests/support` and run the
affected conformance gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `image_sampling_hint_conformance.rs` adopted shared readback/pixel
helpers while retaining explicit render-target setup.
