# Renderer Modularity (Fearless Refactor v1) — Surface Inventory

Status: Draft

Last updated: 2026-03-12

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

## Deferred review candidates

The following exports also have low or niche first-party usage, but are deferred from the first
shrink slice because they are closer to diagnostics or structured public output:

- `AdapterCapabilities`
- `StreamingImageCapabilities`
- `BlurQualityCounters`
- `EffectDegradationCounters`
- `EffectDegradationSnapshot`
- `IntermediatePerfSnapshot`
- `RenderPerfSnapshot`
- `SvgPerfSnapshot`
- `WgpuAllocatorReportSummary`
- `WgpuAllocatorReportTopAllocation`
- `WgpuInitAttemptSnapshot`

These may still move out of the default facade later, but they need a more explicit decision on
how much diagnostics depth `crates/fret-render` is supposed to expose by default.

## Current v1 Recommendation

1. Keep buckets A through E in the default facade.
2. Remove the first shrink candidates from the default facade now.
3. Leave deferred review candidates in place until the diagnostics story is explicitly closed.
4. Keep `crates/fret-render-wgpu` itself broader for now; shrink the default facade first.
