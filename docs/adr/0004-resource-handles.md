# ADR 0004: Resource Handles and Ownership

Status: Accepted

## Context

The editor will render text, icons/images, and embedded engine viewports. We want:

- UI core to be backend-agnostic,
- renderer to own GPU resources,
- a stable ID-based contract for cross-crate communication.

## Decision

Define stable IDs in `fret-core`:

- `ImageId`, `FontId`, `TextBlobId`, `RenderTargetId`, etc.

Ownership rules:

- `fret-render` owns the actual GPU resources and maps IDs to backend handles.
- UI widgets reference IDs only (no `wgpu::TextureView` in UI).
- The engine integration registers external textures/render targets and receives an ID usable by UI.

### Lifetime and eviction

Resource lifetimes must be explicit and compatible with long-lived editors:

- Handle-based resources may be reference-counted or explicitly retained/released.
- Releases are best-effort and can be deferred; dropping a handle does not imply immediate GPU destruction.
- Eviction/GC runs at a well-defined synchronization point (recommended: during the app/platform “effects flush” loop).

### Budgets and observability

Define budget-aware policies for:

- atlas textures (icons, glyphs),
- text blob caches,
- transient per-frame buffers.

Expose counters suitable for debug overlays and profiling (bytes used, hit rates, eviction counts).

## Consequences

- Backend swaps (wgpu → WebGPU) do not leak into UI APIs.
- Engine-hosted vs editor-hosted `Device/Queue` can be supported by `fret-render` without changing UI.
- Resource behavior becomes predictable in production editor workloads (no accidental unbounded growth).

## Future Work

- Implement lifetime/eviction policies (atlas GC, text blob caching) using the chosen “flush point”.
- Define resource budgets and debug counters.
- Decide how external resources are synchronized across engine/UI submissions.
