# UI Assets image loading v1 (ViewCache-safe, query-optional)

## Problem statement

In `apps/fret-ui-gallery`, the Card "event cover" image (`assets/textures/test.jpg`) can fail to appear without any obvious
error surface. The root cause is not "JPEG decode" or "object-fit math" alone: it is an **observability and invalidation**
issue when `ViewCache` reuses a subtree and the image-loading state machine relies on re-running closures to advance.

We need an ecosystem-level image loading helper that:

- Works with `ViewCache` reuse (cached subtree still updates when async decode/GPU-ready completes).
- Does **not** require `fret-query` (query is an ecosystem convenience, not a hard dependency).
- Keeps mechanism vs policy boundaries healthy: `crates/fret-ui` stays a stable contract surface, while image loading
  pipelines live in ecosystem.

## Goals

- Provide a GPUI-style `use_image_source_state` that is **safe under `ViewCache`**.
- Decode off-thread via the runner-provided `DispatcherHandle` (ADR 0175).
- Register images via flush-point effects / stable `ImageId` (ADR 0004).
- Surface useful runtime diagnostics (status + error), without forcing users into diag scripts.
- Keep dependencies minimal and layered:
  - `fret-ui-assets` (ecosystem) owns policy/ergonomics.
  - `fret-asset-cache` (ecosystem) owns cache state machines and budgets.
  - `crates/fret-ui` remains mechanism-only.

## Non-goals (v1)

- A full "CSS/DOM-style asset URL resolver" (packaging, VFS, http fetch).
- A mandatory `fret-query` integration (we keep it optional).
- Perfect shadcn visual parity for all image recipes (this workstream focuses on loading + invalidation correctness).

## Design: ViewCache-safe invalidation

### Why redraw is not enough

When `ViewCache` reuses a subtree, it may skip executing the subtree closure and instead "touch" previously observed
dependencies. If the image loader only schedules work and then calls `request_redraw`, the cached subtree can remain stuck
because it never re-observes a changing dependency.

### Signal model per request

`fret-ui-assets` introduces a tiny per-request signal model:

- `Model<ImageSourceUiSignal { epoch }>`
- Decode completion and GPU-ready transitions bump `epoch`.
- UI callers observe this model (cheap) so the cached subtree becomes invalidated when async work completes.

This keeps the mechanism clean: we do not add a new UI contract surface; we only ensure the ecosystem helper emits an
observable dependency that integrates with the existing `ViewCache` invalidation model.

## Integration points

- UI layer (`ElementContext` sugar):
  - `cx.use_image_source_state(source)` returns `ImageSourceState { image, status, intrinsic_size_px, error }`
  - The sugar is feature-gated as `fret-ui-assets/ui` and is view-cache-safe by default.
- App/driver boundary:
  - The runner must deliver `Event::ImageRegistered` / `Event::ImageRegisterFailed` for effects emitted by the cache.
  - `UiAssets::handle_event` must be wired in the app event pipeline (bootstrap does this behind a feature flag).

## Diagnostics strategy (v1)

- A UI-visible debug overlay in the UI Gallery Card preview can be enabled via:
  - `FRET_UI_GALLERY_DEBUG_IMAGE_LOADING=1`
- Tracing:
  - `image_source: missing DispatcherHandle global (decoding disabled)` (warn once)
  - `image_source: decode failed` (warn)
  - `image_asset_cache: ImageRegistered missing token mapping` (warn)

## Evidence anchors

- Card preview: `apps/fret-ui-gallery/src/ui/previews/gallery/atoms/card.rs`
- Image source state machine + signal model: `ecosystem/fret-ui-assets/src/image_source.rs`
- Event driving: `ecosystem/fret-ui-assets/src/ui_assets.rs`
- Cache state machine: `ecosystem/fret-asset-cache/src/image_asset_cache.rs`

## Follow-ups

See:

- `docs/workstreams/ui-assets-image-loading-v1-todo.md`
- `docs/workstreams/ui-assets-image-loading-v1-milestones.md`

