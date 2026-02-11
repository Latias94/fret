# ADR 0124: Streaming Image Update Effects and Metadata (v1)

Status: Proposed

## Context

ADR 0119 establishes that Fret supports streaming surfaces without owning decoding by providing a portable
`ImageId` update pathway. ADR 0121 adds upload budgets and backpressure.

What remains unspecified is the concrete, stable **data model** used to update streaming images:

- which `Effect` variants exist,
- how updates describe pixel formats (RGBA + reserved YUV),
- how stride and partial (dirty-rect) updates are represented,
- which color metadata is required for correct compositing (ADR 0040),
- how coalescing/backpressure keys are defined,
- whether/when acknowledgements are emitted.

If this is not locked, implementations will drift and later changes will become breaking for apps and plugins.

Related ADRs:

- Streaming ingestion overview: `docs/adr/0119-streaming-images-and-video-surfaces.md`
- Upload budgets/backpressure: `docs/adr/0121-streaming-upload-budgets-and-backpressure-v1.md`
- Color/compositing rules: `docs/adr/0040-color-management-and-compositing-contracts.md`
- Effects queue model: `docs/adr/0001-app-effects.md`

## Decision

### 1) Add explicit streaming update `Effect` variants (v1)

Introduce new `Effect` variants in `crates/fret-runtime` for streaming updates to an existing `ImageId`.

The variants are intentionally “data-only” and are drained by the runner (ADR 0001 / ADR 0034).

#### 1.1) Baseline: RGBA8 update

Proposed variant (shape is normative; naming may be adjusted to match code style):

- `Effect::ImageUpdateRgba8 {`
  - `window: Option<AppWindowId>,`
  - `token: ImageUpdateToken,`
  - `image: ImageId,`
  - `stream_generation: u64,`
  - `width: u32, height: u32,`
  - `update_rect_px: Option<RectPx>,`
  - `bytes_per_row: u32,`
  - `bytes: Vec<u8>,`
  - `color_info: ImageColorInfo,`
  - `alpha_mode: AlphaMode,`
`}`

Notes:

- `bytes` stores the updated region. If `update_rect_px` is `None`, it is the full frame.
- `bytes_per_row` is for the payload layout (not necessarily equal to `width * 4`).
- `window` is optional to allow sources that are not window-bound; when present, it helps the runner decide which
  window to redraw.

#### 1.2) Reserved: NV12 / I420 updates (planar YUV)

v1 reserves explicit variants for common decoder outputs so we can add efficient paths without redesign:

- `Effect::ImageUpdateNv12 { ... }`
  - `stream_generation: u64`
  - Y plane: `y_bytes_per_row`, `y_plane: Vec<u8>`
  - UV plane: `uv_bytes_per_row`, `uv_plane: Vec<u8>`
  - `update_rect_px: Option<RectPx>` (applies to luma; chroma rect derived by subsampling rules)
  - `color_info: ImageColorInfo` (matrix/range/transfer)
  - `alpha_mode: AlphaMode` (typically opaque)

- `Effect::ImageUpdateI420 { ... }`
  - `stream_generation: u64`
  - Y/U/V planes and per-plane stride
  - same metadata fields

Implementation note:

- v1 may implement only `ImageUpdateRgba8` initially, but the API surface should include these variants to avoid
  future breaking changes.
- YUV variants are explicitly designed to allow capability-gated “fast paths” (ADR 0122), such as GPU-assisted
  NV12 conversion or future zero-copy imports, without changing the update contract.

### 2) Define `ImageColorInfo` (minimum metadata for correctness)

Add an explicit metadata struct (in a portable crate, likely `fret-core`) describing how to interpret bytes:

- `ImageColorInfo {`
  - `encoding: ImageEncoding,`
  - `range: ColorRange,`
  - `matrix: YuvMatrix,`
  - `primaries: ColorPrimaries,`
  - `transfer: TransferFunction,`
  - `chroma_siting: Option<ChromaSiting>,`
`}`

Minimum v1 expectations:

- RGBA inputs: `encoding` is `Srgb` or `Linear` (consistent with existing `ImageColorSpace` in renderer code).
- YUV inputs: `range` + `matrix` required; `primaries/transfer` may default to SDR if not provided.

Sampling and conversion requirements:

- The renderer must produce linear values before compositing (ADR 0040).
- sRGB sampling must use sRGB views/formats where available; otherwise apply transfer explicitly.

### 3) Define `AlphaMode` and opacity defaults

Add a small enum:

- `AlphaMode::Opaque`
- `AlphaMode::Premultiplied`
- `AlphaMode::Straight` (optional; may be normalized by the runner/renderer)

v1 recommendation:

- Prefer `Opaque` for video frames.
- Prefer `Premultiplied` for UI/compositor-friendly RGBA.

### 4) Coalescing key and semantics (latest-wins)

Coalescing and backpressure must be deterministic (ADR 0119 / ADR 0121).

Normative key for v1:

- Coalesce by `(image, stream_generation)`

Where:

- `stream_generation: u64` is a required field on update effects and increments when the stream resets (seek,
  format change, source change). This prevents stale pending updates from an earlier stream from being applied
  after a reset.

`stream_generation=0` is valid and means no explicit reset semantics; apps that need robust seek/reset behavior
should increment it.

### 5) Size/format changes: stable `ImageId`, replace underlying storage

Streaming updates may change:

- width/height,
- pixel format (RGBA ↔ YUV),
- color encoding.

The contract allows keeping the same `ImageId` while the renderer replaces underlying GPU storage/view.

Important:

- UI layout must not implicitly depend on image “intrinsic size”. Components should size/fit images explicitly.
- A size change is purely a visual change unless the app chooses to adjust layout.

### 6) Optional acknowledgements (debuggable, capability-gated)

To support debugging and editor tooling, the system may optionally emit acknowledgements:

- `Event::ImageUpdateApplied { token: ImageUpdateToken, image: ImageId }`
- `Event::ImageUpdateDropped { token: ImageUpdateToken, image: ImageId, reason: ImageUpdateDropReason }`

This must be capability-gated to avoid flooding the event loop in normal playback.

## Consequences

- Apps can build video players, remote previews, and timeline scrubbing without owning renderer internals.
- Future efficient paths (YUV + zero-copy imports) can be added without breaking the API.
- Backpressure behavior is consistent across platforms and workloads.

## Validation / Acceptance Criteria

Implementation is considered conformant when:

- Updates are latest-wins coalescible by `(ImageId, stream_generation)` and behavior is deterministic.
- `bytes_per_row` and `update_rect_px` are supported (or rejected with a clear error) without undefined behavior.
- The renderer composites streamed pixels correctly with the declared `ImageColorInfo` and `AlphaMode` (ADR 0040).

## References

- Streaming surfaces overview: `docs/adr/0119-streaming-images-and-video-surfaces.md`
- Upload budgets/backpressure: `docs/adr/0121-streaming-upload-budgets-and-backpressure-v1.md`
- Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`
