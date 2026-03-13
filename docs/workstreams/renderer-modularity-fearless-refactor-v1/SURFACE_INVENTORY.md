# Renderer Modularity (Fearless Refactor v1) — Surface Inventory

Status: In progress

Last updated: 2026-03-13

Related:

- Purpose: `docs/workstreams/renderer-modularity-fearless-refactor-v1/README.md`
- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/renderer-modularity-fearless-refactor-v1/TODO.md`

## Goal

This note records the current default-renderer surface and the first-party usage patterns we found
before shrinking the facade further.

The point is not to prove that every export is wrong. The point is to avoid accidental public API
decisions while modularization is in flight.

## Audit Method

As of 2026-03-12:

- source of truth for backend exports:
  - `crates/fret-render-wgpu/src/lib.rs`
- source of truth for default facade exports:
  - `crates/fret-render/src/lib.rs`
- first-party consumer scan:
  - workspace-wide `rg` over `crates/`, `ecosystem/`, and `apps/`
  - excluding `crates/fret-render-wgpu/*` when measuring downstream consumers

## Current Consumer Buckets

### A. Core runtime / bootstrap surface

These are the most broadly used first-party exports and should remain part of the default facade
unless we intentionally redesign the public story.

- `Renderer`
- `RenderSceneParams`
- `WgpuContext`
- `SurfaceState`
- `RenderError`
- `ClearColor`

Primary consumers:

- `crates/fret-launch`
- `apps/fret-examples`
- stress/demo apps under `apps/`

### B. Capability / diagnostics surface

These are actively used by first-party runners, demos, or diagnostics flows.

- `RendererCapabilities`
- `WgpuAdapterSelectionSnapshot`
- `RendererPerfFrameStore`
- `RendererPerfFrameSample`
- `WgpuHubReportCounts`
- `WgpuHubReportFrameSample`
- `WgpuHubReportFrameStore`
- `WgpuAllocatorReportFrameSample`
- `WgpuAllocatorReportFrameStore`

Primary consumers:

- `crates/fret-launch`
- `ecosystem/fret-bootstrap`
- selected demos / cookbook examples

### C. Render-target and external-ingest contracts

These are used by embedded viewport, video import, or external texture paths and belong to the
default story today.

- `RenderTargetDescriptor`
- `RenderTargetMetadata`
- `RenderTargetColorSpace`
- `RenderTargetColorEncoding`
- `RenderTargetColorPrimaries`
- `RenderTargetColorRange`
- `RenderTargetMatrixCoefficients`
- `RenderTargetTransferFunction`
- `RenderTargetOrientation`
- `RenderTargetRotation`
- `RenderTargetAlphaMode`
- `RenderTargetIngestStrategy`

Primary consumers:

- `crates/fret-launch`
- `ecosystem/fret`
- `apps/fret-examples`
- `apps/fret-cookbook`

### D. Image upload / mutation helpers

These remain part of the current golden path for external textures, sampled images, and demo code.

- `ImageDescriptor`
- `ImageColorSpace`
- `UploadedRgba8Image`
- `create_rgba8_image_storage`
- `upload_rgba8_image`
- `write_rgba8_texture_region`

Primary consumers:

- `crates/fret-launch`
- `apps/fret-examples`
- `apps/fret-cookbook`

### E. Viewport overlay support

This remains actively used and should stay visible from the default facade.

- `viewport_overlay`

Primary consumers:

- `crates/fret-launch`
- `apps/fret-examples/src/gizmo3d_demo.rs`

## First shrink candidates (default facade)

The following exports had no first-party consumers outside the facade itself during the 2026-03-12
workspace scan and are therefore the safest initial shrink candidates for `crates/fret-render`:

- `ImageRegistry`
- `RenderTargetRegistry`
- `CachedSvgImage`
- `SvgImageCache`
- `SvgRasterKind`
- `SvgRenderer`
- `UploadedAlphaMask`
- `UploadedRgbaImage`
- `SMOOTH_SVG_SCALE_FACTOR`

Rationale:

- they look like backend implementation details or low-level helper surfaces,
- they are not part of the first-party runner/bootstrap story,
- and they are not required by the current demo / cookbook / launch paths.

## Store-vs-snapshot closure (2026-03-13)

The 2026-03-13 rescan closed the "stores vs deeper diagnostics values" question for the default
facade:

- keep diagnostics/report stores and their immediate sample/count wrapper types on `crates/fret-render`
- move zero-direct-consumer advanced perf/init value snapshots out of the default facade

Why the stores stay:

- `RendererPerfFrameStore` / `RendererPerfFrameSample` are used by `crates/fret-launch`,
  `ecosystem/fret-bootstrap`, and `apps/fret-examples`
- `WgpuHubReportCounts` / `WgpuHubReportFrameStore` / `WgpuHubReportFrameSample` are used by
  `crates/fret-launch` and `ecosystem/fret-bootstrap`
- `WgpuAllocatorReportFrameStore` / `WgpuAllocatorReportFrameSample` are used by
  `crates/fret-launch` and `ecosystem/fret-bootstrap`

## Advanced value snapshots retired from the default facade

The following advanced diagnostics/perf value types had no direct first-party consumers outside
`crates/fret-render*` during the 2026-03-13 rescan and have now been removed from
`crates/fret-render`:

- `BlurQualitySnapshot`
- `EffectDegradationSnapshot`
- `RenderPerfSnapshot`
- `IntermediatePerfSnapshot`
- `SvgPerfSnapshot`
- `WgpuInitDiagnosticsSnapshot`

These values still exist in the backend and can still be observed indirectly through public parent
surfaces such as `RendererPerfFrameSample`, `Renderer::take_*_snapshot(...)`, and
`WgpuContext::init_diagnostics`. The decision here is only that naming these advanced value types
directly is no longer part of the stable default-facade story.

## Nested detail structs retired from the default facade

The following leaf/detail structs had no first-party consumers outside `crates/fret-render*` during
the 2026-03-13 rescan and have now been removed from `crates/fret-render`:

- `AdapterCapabilities`
- `StreamingImageCapabilities`
- `BlurQualityCounters`
- `EffectDegradationCounters`
- `WgpuAllocatorReportSummary`
- `WgpuAllocatorReportTopAllocation`
- `WgpuInitAttemptSnapshot`

Rationale:

- they are detail rows nested inside still-public parent snapshots/stores,
- they are not part of the primary default-facade teaching surface,
- and first-party callers did not need source changes after the shrink.

## Stable v1 Facade Closure (2026-03-13)

The stable default-facade contract for v1 is now intentionally described as:

1. Buckets A through E are the public default-facade story.
2. Nested diagnostics detail structs may stay backend-only even when their parent snapshots/stores
   remain on the default facade.
3. Advanced perf/init value snapshots may stay backend-only even when parent stores or convenience
   surfaces still expose those values indirectly.
4. `crates/fret-render/tests/facade_surface_snapshot.rs` is the external compile-time gate for the
   chosen buckets.
5. `crates/fret-render/src/lib.rs` compile-fail doctests now guard backend-only advanced snapshot
   names from accidentally re-entering the default facade.
6. `crates/fret-render/src/lib.rs` and `docs/crate-usage-guide.md` are the public prose anchors
   for this facade story.

## Current v1 Recommendation

1. Keep buckets A through E in the default facade.
2. Remove the first shrink candidates from the default facade now.
3. Keep diagnostics/report stores on the default facade where first-party runners/tooling use them.
4. Keep advanced perf/init value snapshots backend-specific unless a real default-facade consumer
   appears.
5. Keep `crates/fret-render-wgpu` itself broader for now; shrink the default facade first.
