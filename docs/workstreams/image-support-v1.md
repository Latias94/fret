Status: Draft (notes only; ADRs remain the source of truth)

This workstream covers **image support as a first-class UI primitive** in Fret, with an explicit
focus on editor-grade workloads:

- shadcn-style components (Avatar, cards, media rows) that expect `object-fit: cover/contain`,
- thumbnails and large scrolling lists (cache + budgets),
- video playback surfaces (streaming `ImageId` updates),
- engine/viewports (`RenderTargetId`) without forcing the UI layer to own GPU pipelines.

Related ADRs (existing):

- Resource handles + flush point: `docs/adr/0004-resource-handles.md`
- Streaming images + video surfaces: `docs/adr/0121-streaming-images-and-video-surfaces.md`
- Streaming update model + metadata: `docs/adr/0126-streaming-image-update-effects-and-metadata-v1.md`
- Renderer capabilities + optional fast paths: `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`
- Color/compositing contracts: `docs/adr/0040-color-management-and-compositing-contracts.md`

Related ADRs (planned / in progress):

- Image `object-fit` for `SceneOp::Image`: `docs/adr/1170-image-object-fit-for-sceneop-image-v1.md`

Tracking:

- `docs/workstreams/image-support-v1-todo.md`

## Current state (baseline)

Fret already supports:

- Drawing images by stable ID: `SceneOp::Image` / `SceneOp::ImageRegion`.
- Register/unregister image resources at a flush point via effects (`Effect::ImageRegisterRgba8`,
  `Effect::ImageUnregister`).
- Streaming image updates (`Effect::ImageUpdateRgba8/Nv12/I420`) with coalescing, budgets, and
  optional acknowledgements (ADR 0121/0123/0126).
- Viewport surfaces (`RenderTargetId` + `SceneOp::ViewportSurface`) for GPU-native pipelines.

Gaps that create “future rewrite” pressure:

1. No stable `object-fit` semantics for images (Avatar and card images stretch).
2. No stable “image metadata query” seam (intrinsic size/aspect ratio is not observable without
   app-owned side channels).
3. No canonical ecosystem “img(source)” story (decode/load/caching sits in ad-hoc app code today).

## Goals

1. Make **image fit semantics** stable and shared across:
   - still images (`SceneOp::Image`),
   - viewport surfaces (`SceneOp::ViewportSurface`),
   - and ecosystem components (shadcn).
2. Preserve Fret’s layering invariants:
   - no `wgpu` types in `fret-core` / `fret-ui`,
   - resources managed at a flush point; UI only holds IDs (ADR 0004),
   - decoding/network/media engines remain app-owned (ADR 0121).
3. Provide a path to **gpui-like ergonomics** in ecosystem crates without baking policy into
   `fret-ui`.
4. Keep streaming/video surfaces correct and predictable:
   - latest-wins updates,
   - deterministic backpressure,
   - explicit color/alpha semantics (ADR 0040/0126).

## Non-goals

- Framework-owned codecs, audio, network streaming, or a “media engine”.
- Forcing all video to go through `RenderTargetId` (both ingestion paths remain valid; ADR 0121).
- Implicit layout based on “intrinsic image size” (components should be explicit and predictable).

## Proposed direction (recommended)

### A) Lock `object-fit` for images as a core contract

Add a stable fit field for `SceneOp::Image` (and the declarative `Image` element surface), aligned
with the existing `ViewportFit` vocabulary:

- `Stretch` (CSS `fill`): current behavior.
- `Contain`: preserve aspect ratio; letterbox/pillarbox inside the rect.
- `Cover`: preserve aspect ratio; crop to fill the rect (center-crop by default).

Key design rule:

- `SceneOp::ImageRegion` remains “caller-specified UV”; it does not apply fit logic.
  (Fit + explicit UV becomes ambiguous and makes future behavior hard to reason about.)

Decision gate: whether we reuse `ViewportFit` directly or introduce a shared `ObjectFit` type that
both viewport and image paths reference.

### B) Add an explicit image metadata query seam (optional, but likely needed)

To support aspect-ratio-aware layouts and ecosystem components that want intrinsic sizing without
app side channels, introduce a portable query surface:

- `trait ImageService` in `fret-core` (plumbed through `UiServices`), e.g.
  - `image_size(ImageId) -> Option<(u32, u32)>`
  - optional: `image_alpha_mode(ImageId) -> Option<AlphaMode>`

This is intentionally *metadata only* (not loading/decoding).

### C) Ecosystem `img(source)` story (deferred until A is locked)

After fit semantics are locked and shadcn usage is correct, add an ecosystem crate that provides a
gpui-style experience:

- `ImageSource` (path/url/bytes/custom loader),
- async decode/fetch in app-owned tasks (`fret-executor`),
- reuse `fret-ui-assets` / `fret-asset-cache` for budgeting + `use_asset`-style caching,
- component-friendly state (`Idle/Loading/Loaded/Error`) and skeleton/fallback recipes.

This stays out of `fret-ui` and is capability-gated for wasm vs native.

## Video / RenderTarget considerations (how this avoids a later rewrite)

Fret should keep *both* ingestion paths (ADR 0121):

1) `RenderTargetId` + `SceneOp::ViewportSurface` for GPU-native render graphs and engine viewports.
2) `ImageId` + streaming update effects for decoder-owned frames (video, camera, remote desktop).

The “avoid big rewrite” lever is shared fit semantics:

- Viewport surfaces already have `ViewportFit`.
- Images need the same stable fit vocabulary so video players, thumbnails, and avatars do not
  re-implement crop logic in every component crate.

## Validation strategy (how we keep this stable)

- Add renderer-facing conformance tests for fit mapping and UV crop math.
- Add shadcn regression gates:
  - avatar image should be center-cropped (no stretching),
  - cover/contain behavior matches web goldens where applicable.
- Add at least one scripted diag scenario (optional): image-in-card + overlay controls + resize.
