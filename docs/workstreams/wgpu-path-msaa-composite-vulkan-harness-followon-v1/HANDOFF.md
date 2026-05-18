# WGPU Path MSAA Composite Vulkan Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The Vulkan path-MSAA composite smoke test uses shared WGPU test support for raw
texture readback and byte sampling while keeping BGRA naming and Vulkan/MSAA assertions.

## Completed

- Removed duplicated local `read_texture_rgba8` and `pixel_bgra` helper bodies from
  `crates/fret-render-wgpu/tests/path_msaa_composite_vulkan.rs`.
- Preserved local `pixel_bgra` alias, Vulkan backend guard, path-MSAA setup, explicit BGRA output
  format, and red/green visibility assertions.
- Recorded gates and closeout evidence.

## Continue Policy

Do not reopen this lane for the Vulkan MSAA visibility test. Start a separate narrow follow-on.
