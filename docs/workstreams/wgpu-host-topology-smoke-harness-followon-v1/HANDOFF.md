# WGPU Host Topology Smoke Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The host-provided GPU topology smoke test uses shared WGPU test support for
readback and pixel sampling while keeping direct adapter/device/queue setup local.

## Completed

- Removed duplicated local `read_texture_rgba8` and `pixel_rgba` helpers from
  `crates/fret-render-wgpu/tests/host_provided_gpu_topology_smoke.rs`.
- Preserved direct host-provided GPU object requests, capability assertions, explicit render target
  setup, and renderer construction path.
- Recorded gates and closeout evidence.

## Continue Policy

Do not reopen this lane for Vulkan or MSAA cleanup. Start separate narrow follow-ons for those tests.
