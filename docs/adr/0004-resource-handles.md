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

## Implementation Notes (Current)

### Effect-driven image registration (portable)

To keep `fret-ui` and component crates backend-agnostic, raw image bytes must cross the backend boundary via
effects and be registered by the runner/renderer at a well-defined synchronization point (the effects drain loop).

Current minimal contract:

- Token: `fret_core::ImageUploadToken` (allocated by the host via `UiHost::next_image_upload_token()`).
- Request: `fret_runtime::Effect::ImageRegisterRgba8 { window, token, width, height, bytes, color_info, alpha_mode }`.
- Success: `fret_core::Event::ImageRegistered { token, image, width, height }`.
- Failure: `fret_core::Event::ImageRegisterFailed { token, message }`.
- Release: `fret_runtime::Effect::ImageUnregister { image }` (best-effort).

Notes:

- This matches the GPUI-style principle that resources are managed at a flush point and UI code only holds stable IDs.
- Higher-level asset caches (key → async load → register → notify redraw) are expected to live above this minimal
  primitive (similar to GPUI’s `use_asset`), but the core boundary stays effects-based and portable.
- `color_info` / `alpha_mode` are stable metadata types (`fret_core::ImageColorInfo`, `fret_core::AlphaMode`) aligned with ADR 0124.

Zed/GPUI reference (non-normative):

- GPUI uses stable IDs for cached images and drives image loading/caching above the low-level draw
  API (illustrative of “ID in UI, resource in renderer” separation):
  `repo-ref/zed/crates/gpui/src/assets.rs` (`ImageId`, `RenderImage`),
  `repo-ref/zed/crates/gpui/src/asset_cache.rs`.
