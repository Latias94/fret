# WGPU Host Topology Smoke Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`host_provided_gpu_topology_smoke.rs` still duplicated RGBA8 readback and pixel helper functions even
though the test's unique contract is the host-provided adapter/device/queue topology, not readback
mechanics.

This lane removes only the duplicated readback helpers while preserving the direct host-provided GPU
setup, capability snapshot assertions, explicit render target, and renderer construction path.

## Assumptions First

- Confident: this is a helper-migration refactor only. Evidence: the file currently defines local
  `read_texture_rgba8` and `pixel_rgba`, while the host topology contract is encoded by
  `request_engine_hosted_gpu_objects`, `RendererCapabilities::from_adapter_device`, and
  `Renderer::new(&adapter, &device)`. If wrong, the host topology smoke gate should fail.
- Confident: the test should not adopt `render_scene_rgba8`. Evidence: the shared render helper
  requires `WgpuContext`, while this test intentionally proves direct adapter/device/queue usage.
- Likely: ADR alignment docs do not need content changes because the evidence file remains the same
  and only delegates readback mechanics to shared test support. If wrong, refresh ADR 0010 / renderer
  modularity evidence anchors.

## Target State

- `host_provided_gpu_topology_smoke.rs` imports `tests/support::{read_texture_rgba8, pixel_rgba}`.
- Direct host-provided GPU setup, capability assertions, explicit render target setup, and scene
  assertions remain unchanged.
- The lane closes after this narrow migration.

## Out Of Scope

- Changing `WgpuContext`, renderer construction, capability snapshots, adapter/device requests, or
  downlevel requirements.
- Replacing the explicit render target with a `WgpuContext` helper.
- Migrating Vulkan or MSAA tests.

## First Slice

`WHT-010`: migrate duplicated host-topology readback/pixel helpers onto `tests/support` and run the
affected smoke gate.

## Closure Policy

Close this lane once the code migration and gates pass.

## Closure

Closed on 2026-05-18 after `host_provided_gpu_topology_smoke.rs` adopted shared readback/pixel
helpers while retaining the direct host-provided adapter/device/queue path.
