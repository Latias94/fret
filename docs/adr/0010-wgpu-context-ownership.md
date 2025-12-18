# ADR 0010: WGPU Context Ownership (Host-Provided GPU Context)

Status: Accepted

## Context

Fret must support editor integration with multiple Rust game engines built on `wgpu`.

Two common hosting topologies exist:

1. **Editor-hosted GPU**: the UI framework creates `wgpu::Instance/Adapter/Device/Queue`.
   The engine renders using the shared `Device/Queue`.
2. **Engine-hosted GPU**: the engine creates `wgpu::Instance/Adapter/Device/Queue`.
   The UI framework renders into platform surfaces using the engine’s `Device/Queue`.

If we choose only one topology, we risk forcing downstream users into an integration style that
may not match their engine architecture, leading to a later rewrite.

## Decision

### 1) Host-provided `WgpuContext` is the primary contract

Fret treats a `WgpuContext` (instance + adapter + device + queue) as a **host-provided** input to
the platform runner / renderer pipeline.

This enables both hosting topologies:

- **Editor-hosted**: Fret constructs the `WgpuContext` and passes it to the engine.
- **Engine-hosted**: the engine constructs the `WgpuContext` and passes it to Fret.

### 2) Surface creation is owned by the platform layer, but depends on the same `Instance`

On desktop, the platform layer owns OS windows and their presentable surfaces.

However, surface creation (`Instance::create_surface`) must be done with the same `Instance`
that the renderer/device belong to. Therefore:

- If the host provides a `WgpuContext`, the platform layer uses `context.instance` to create surfaces.
- The platform layer must not silently create a second `wgpu::Instance` “on the side”.

### 3) Engine viewports remain handle-based

Engine-rendered textures are registered as `RenderTargetId` via renderer-owned registries.
This stays unchanged across both hosting topologies.

## Consequences

- Both editor-hosted and engine-hosted integrations remain first-class, avoiding lock-in.
- Multi-window tear-off remains possible because surface creation stays centralized in the platform layer.
- wasm/WebGPU remains feasible: the platform layer adapts “surface” creation to web canvases while keeping
  the same `WgpuContext` contract.

## Future Work

- Define an explicit “context injection” API for runners (desktop/web) rather than hard-wiring context creation.
- Add guidance for advanced engines (multiple devices, multiple queues, headless rendering).
- Document synchronization expectations when sharing a `Device/Queue` between engine and UI.

