# ADR 0007: Engine Viewports via RenderTargetId

Status: Accepted

## Context

An editor needs multiple engine viewports:

- embedded in docked panels,
- across multiple OS windows (tear-off),
- with UI overlays (gizmos, selection rectangles) on top.

Fret must support a wgpu-based game engine integration without leaking `wgpu` types into `fret-core` / `fret-ui`.

## Decision

Use `RenderTargetId` (from `fret-core`) as the stable handle for “engine frames”:

- UI emits `SceneOp::ViewportSurface { target: RenderTargetId, rect, opacity, ... }`.
- The renderer owns the registry that resolves `RenderTargetId` to GPU resources.

### Primary integration path (recommended)

**Editor-hosted GPU context**:

- Fret creates `wgpu::Instance/Adapter/Device/Queue` in the platform layer.
- The engine is given shared access to `Device/Queue` and produces render targets on it.
- Viewports are displayed with zero-copy sampling of the engine-produced texture.

### Secondary integration path (optional)

**Engine-hosted GPU context**:

- The engine provides `Device/Queue` (and possibly instance/adapter).
- Fret attaches surfaces and UI rendering on top.

Both paths must preserve the same `RenderTargetId` contract.

## Render Target Contract (Registry)

The renderer registry API is wgpu-specific and lives in `fret-render`.

Minimum required metadata:

- texture view handle (sampled),
- size (pixels),
- format / color space expectations (sRGB vs linear),
- sample count (MSAA resolve requirements, if any).

### Invariants

- `RenderTargetId` is opaque to `fret-ui`.
- Missing targets are best-effort: if a target is not found, the renderer skips the op.
- The engine must update or re-register targets on resize.

## Consequences

- Multi-viewport becomes a first-class feature without coupling UI to engine internals.
- Docking + tear-off windows can show the same target in multiple places.
- Future wasm/WebGPU portability remains possible because the core contract is wgpu-free.

## Future Work

- Resolve targets: support MSAA render targets by registering a resolved view.
- Overlay ordering: define how viewport surfaces interact with clip stacks and draw order.

