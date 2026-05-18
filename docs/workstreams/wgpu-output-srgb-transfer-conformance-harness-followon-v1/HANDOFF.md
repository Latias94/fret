# WGPU Output sRGB Transfer Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The output sRGB transfer conformance test uses shared WGPU test support for
RGBA8 readback and pixel sampling while keeping explicit output texture setup and sRGB transfer
assertions local.

## Completed

- Removed duplicated local `read_texture_rgba8` and `pixel_rgba` helpers from
  `crates/fret-render-wgpu/tests/output_srgb_transfer_conformance.rs`.
- Preserved explicit `Rgba8Unorm` output texture setup, transfer math, quantization expectation, and
  pixel assertions.
- Recorded gates and closeout evidence.

## Continue Policy

Treat this as a narrow format/transfer follow-on. Do not expand it to metadata, backend, or platform
specific tests.
