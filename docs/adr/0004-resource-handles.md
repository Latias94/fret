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

## Consequences

- Backend swaps (wgpu → WebGPU) do not leak into UI APIs.
- Engine-hosted vs editor-hosted `Device/Queue` can be supported by `fret-render` without changing UI.

## Future Work

- Define lifetimes and eviction policies (atlas GC, text blob caching).
- Decide how external resources are synchronized across engine/UI submissions.

