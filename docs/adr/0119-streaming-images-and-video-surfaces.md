# ADR 0119: Streaming Images and Video Surfaces (Frame Input Without Decoder Ownership)

Status: Proposed

## Context

Fret is a UI framework, not a media engine. Apps may still need to render:

- engine viewports (`GameView`) inside dock panels,
- local video playback (e.g. FFmpeg on desktop),
- web video playback (e.g. WebCodecs / HTML video on wasm),
- editor-grade timeline scrubbing (many seeks, frame-accurate display).

Fret must support these without taking ownership of:

- audio playback,
- video decoding,
- codec selection,
- network streaming.

Instead, Fret should provide stable, portable **frame input contracts** that let apps feed decoded frames (or
GPU-rendered results) into the UI compositor efficiently and safely, while preserving Fret’s core invariants:

- ordered compositing via `Scene.ops` (ADR 0002 / ADR 0009),
- linear-space compositing and explicit target encoding metadata (ADR 0040),
- event-driven scheduling by default (ADR 0034),
- no `wgpu` types in `fret-core` / `fret-ui` (ADR 0004 / ADR 0092).

Related policies and follow-ups:

- Postprocessing intermediate budgets and degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Streaming upload budgets/backpressure: `docs/adr/0121-streaming-upload-budgets-and-backpressure-v1.md`
- Renderer capability negotiation (optional zero-copy): `docs/adr/0122-renderer-capabilities-and-optional-zero-copy-imports.md`
- Streaming update effect/data model: `docs/adr/0124-streaming-image-update-effects-and-metadata-v1.md`

We already support engine-rendered viewports via `RenderTargetId` + `SceneOp::ViewportSurface` (ADR 0007) and a
submission coordinator model (ADR 0038). What is missing is a first-class contract for **streaming image updates**
that can be driven by external frame sources (video, camera, remote desktop, etc.).

## Decision

### 1) Two ingestion paths: `RenderTargetId` for GPU pipelines, `ImageId` for streaming frames

Fret supports two complementary input paths:

1) **Viewport / render graph output (GPU-native)**
   - Apps/engines record GPU work and render into textures registered as `RenderTargetId`.
   - UI displays it via `SceneOp::ViewportSurface`.
   - Submission remains coordinated by the runner (ADR 0038).

2) **Streaming image frames (decoder-owned)**
   - Apps provide decoded frames (bytes) to the renderer as updates to an existing `ImageId`.
   - UI displays it via `SceneOp::Image` / `SceneOp::ImageRegion` (or equivalent image primitives).

These paths serve different workloads:

- engine viewports and heavy filter graphs: prefer `RenderTargetId`,
- typical video player UI, thumbnails, and timeline scrubbing: prefer streaming `ImageId` updates.

#### Use cases and recommended path (non-normative guidance)

- **Editor `GameView` / engine viewport**: `RenderTargetId` (`SceneOp::ViewportSurface`).
  - Rationale: the engine already has a GPU pipeline / render graph; Fret should only composite its output.
- **Local video playback UI (FFmpeg, desktop)**: start with streaming `ImageId` updates.
  - Rationale: simplest integration; decoder remains app-owned; UI draws an image + overlays controls.
- **Timeline scrubbing / clip thumbnails**: streaming `ImageId` updates.
  - Rationale: frame-accurate seeks benefit from latest-wins and coalescing (no queues).
- **Editor-grade multi-layer video compositing (NLE-class)**: often best as `RenderTargetId`.
  - Rationale: heavy effects, multiple streams, and compositing pipelines map naturally to GPU graphs; UI consumes the final texture.

### 2) Streaming updates are expressed as effects (data), not direct renderer calls

To keep scheduling deterministic and portable across backends (desktop + wasm), streaming frame updates must be
expressed as `Effect`s drained by the app/runner loop (ADR 0001 / ADR 0034).

This keeps all frame ingestion inside the same “fixed-point drain” model as other external I/O.

### 3) Coalescing and backpressure are part of the contract (latest-wins)

Streaming frames can arrive faster than the UI render cadence and must not create unbounded queues.

Contract (v1):

- Frame updates are **coalescible** by `(ImageId, stream_generation)` (locked by ADR 0124).
- The renderer/runner may drop intermediate frames and keep only the most recent pending update per image.
- If an update is dropped, the UI must remain correct (it only affects visual freshness, not state).

This enables:

- 60fps playback without effect-queue blowups,
- editor scrubbing where seeks invalidate prior frames.

#### Why backpressure belongs at the framework boundary

Backpressure here is not a "media feature"; it is a **compositor stability** requirement.
Without a framework-level coalescing rule, it is easy for apps to accidentally build unbounded queues of large
frames (especially when a window is occluded, throttled, or rendering slowly), leading to memory spikes and
non-deterministic latency.

### 4) Pixel formats: baseline set + extensibility

We define a baseline set of pixel formats that the renderer must support for streaming images:

- `Rgba8` (premultiplied alpha expected; linear/sRGB handling per ADR 0040)

And reserve space for planar YUV formats commonly produced by decoders:

- `Nv12` (Y plane + interleaved UV)
- `I420` / `Yuv420p` (Y + U + V planes)

Notes:

- v1 may implement only `Rgba8` first, but the contract should include the YUV variants so we do not need a
  breaking redesign when adding efficient video paths.
- YUV sampling and conversion must produce linear values before compositing (ADR 0040).

### 4.1) Stride and plane layout are first-class for streaming updates

To avoid forcing expensive CPU repacking, streaming update payloads should be able to describe:

- per-plane `bytes_per_row` (stride),
- plane offsets and sizes (for planar formats),
- and a "tight vs padded" layout.

The renderer may still repack internally when required by platform constraints (e.g. `COPY_BYTES_PER_ROW_ALIGNMENT`),
but the public contract should not assume tightly packed rows.

### 4.2) Partial updates (dirty rect) are reserved for remote/interactive sources

Some sources (remote desktop, canvas capture, certain decoders) can provide updates for only a sub-rectangle.
The ingestion contract should reserve the ability to submit partial updates:

- `update_rect: RectPx` (pixel-space rectangle within the image) plus per-plane layouts for that region.

v1 implementation may still choose to repack into a full-frame upload for simplicity, but the public shape should
avoid making full-frame-only a hard constraint.

### 5) Color encoding metadata is explicit

Streaming frame updates must carry enough metadata for correct sampling and conversion:

- range: limited vs full,
- matrix: BT.601 / BT.709 / BT.2020 (at minimum),
- transfer and primaries (v1 may treat frames as display-referred SDR; HDR is future work),
- orientation/rotation (optional; apps may also handle via transforms).

If metadata is missing, the renderer may apply conservative defaults (documented), but correctness requires that
apps provide accurate metadata for non-default sources.

### 6) Scheduling: frame arrival triggers redraw, but only while visible/active

When a new frame update is applied, the renderer/runner should request a redraw for the owning window.

However:

- windows that are idle and not showing any streaming images should remain idle (ADR 0034),
- future optimization: only request redraw when the image is actually referenced in the current `Scene` for that window.

### 7) Capability-gated zero-copy imports (future, optional)

Some platforms provide GPU-native decoded frames (e.g. WebCodecs `VideoFrame` → WebGPU external texture).

We do not require this for v1 portability, but we reserve a capability-gated path:

- “import external texture” as an image source, without copying bytes through the effect queue,
- still referenced in UI by `ImageId` or `RenderTargetId`, never by raw backend handles.

This must live in backend/renderer crates (not `fret-core`) and be explicitly negotiated via capabilities.

## Crate Placement and Implementation Sketch (Non-normative)

This section describes a likely implementation placement consistent with ADR 0092.

### `RenderTargetId` path (GPU-native)

- **App/engine code (outside Fret core crates)**:
  - owns decoding or rendering pipelines,
  - records GPU command buffers targeting textures that will be registered as `RenderTargetId`.
- **Runner / submission coordinator** (`crates/fret-launch`):
  - applies render target registry updates,
  - submits engine command buffers and then UI command buffers (ADR 0038).
- **Renderer** (`crates/fret-render`):
  - maintains the render target registry and metadata (`encoding`, alpha semantics),
  - samples targets via `SceneOp::ViewportSurface` (ADR 0007 / ADR 0040).

### `ImageId` streaming path (decoder-owned)

- **Effect definitions** (`crates/fret-runtime`):
  - add `Effect` variants for streaming updates (see ADR 0124), e.g.:
    - `Effect::ImageUpdateRgba8 { image: ImageId, stream_generation: u64, ... }`
    - `Effect::ImageUpdateNv12 { image: ImageId, stream_generation: u64, ... }`
    - `Effect::ImageUpdateI420 { image: ImageId, stream_generation: u64, ... }`
  - define these updates as coalescible (latest-wins) by `(ImageId, stream_generation)` (ADR 0124).
- **App runtime** (`crates/fret-app`):
  - remains the owner of effect queues and tick/frame scheduling (ADR 0001 / ADR 0034).
- **Runner** (`crates/fret-launch`, and web runner equivalents):
  - drains image update effects,
  - coalesces updates per `(ImageId, stream_generation)` (ADR 0124),
  - uploads to GPU via renderer APIs,
  - requests redraw for the relevant window when appropriate.
- **Renderer** (`crates/fret-render`):
  - provides image storage/registry and efficient update paths:
    - preferred: update bytes into a stable GPU texture when dimensions/format match,
    - fallback: replace the underlying texture/view when size/format changes.

Wasm notes:

- wasm backends may start with byte uploads (`Rgba8`), then later add capability-gated external texture imports
  when the platform supports it (kept out of `fret-core` and negotiated at backend boundaries).

## Consequences

- Apps can build both simple players and editor-grade multi-viewport tools without framework-owned decoders.
- The renderer can provide efficient, bounded frame ingestion semantics (latest-wins).
- wasm support remains a framework property: the app chooses the decoding approach.

## Follow-up Work

- Lock the streaming update data model and metadata vocabulary (done: ADR 0124).
- Add a budget policy for streaming upload bandwidth and staging memory (done: ADR 0121).
- Add optional shared-buffer/zero-copy ingestion paths (capability-gated; see ADR 0122 / ADR 0123).

## References

- Effects queue and scheduling: `docs/adr/0001-app-effects.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Viewports and submission coordinator: `docs/adr/0007-viewport-surfaces.md`, `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`
- Renderer v3 substrate + budgets: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`,
  `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`

### Ecosystem Reference Notes (Why We Consider Formats + Backpressure)

These are non-normative references used to sanity-check the direction.

- **Qt (Qt Multimedia)**: models decoded frames as a reusable `VideoFrame`-like object with explicit pixel formats
  (including common YUV variants) and connects them to rendering surfaces (a video output item). This is a common
  pattern: the framework does not "own decoding", but it *does* standardize frame ingestion and rendering semantics.
- **Flutter**: exposes a `Texture`/texture-registry style integration point so plugins/apps can feed external
  frames to the compositor without the UI owning decoding. This requires coalescing/backpressure to avoid choking
  the render thread when frames arrive faster than render cadence.
- **Skia / modern GPU compositors**: commonly support YUV-backed image inputs and GPU-side YUV→RGB conversion.
  This keeps video playback efficient and avoids forcing every app to do costly CPU conversions.
- **Dear ImGui**: treats textures as opaque IDs (`ImTextureID`). This pushes the responsibility for texture
  updates, synchronization, and any backpressure entirely to the host renderer. Fret chooses to standardize a
  minimal, portable subset of these responsibilities so apps do not reinvent them per platform.

Concrete Fret-driven motivations:

- **Video player UI**: needs a stable way to update a displayed image at a steady cadence without unbounded queues.
- **Editor `GameView` / engine viewport**: should remain a `RenderTargetId` pathway (ADR 0007 / ADR 0038), but
  the editor may still want thumbnail previews, scrubbable clips, or remote-stream surfaces, which fit the
  streaming `ImageId` pathway.
