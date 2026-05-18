# WGPU Image Sampling Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The image sampling conformance test uses shared WGPU test support for RGBA8
readback and pixel sampling while keeping explicit render-target setup local.

## Completed

- Removed duplicated local `read_texture_rgba8` and `pixel_rgba` helpers from
  `crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs`.
- Preserved checkerboard image registration, explicit render-target setup, sampler hint assertions,
  and mixed primitive ordering assertions.
- Recorded gates and closeout evidence.

## Continue Policy

Do not reopen this lane for unrelated conformance families. Start a narrow follow-on for each
remaining family whose helper shape is proven compatible with shared support.

Recommended follow-ons:

- Output sRGB transfer as a format/transfer-specific lane.
- Viewport metadata as a metadata-specific lane.
- Vulkan/MSAA and host topology tests as backend/platform-specific lanes.
