Status: Proposed

## Context

Fret exposes image drawing as portable scene primitives:

- `SceneOp::Image`
- `SceneOp::ImageRegion`

UI and component crates (including `fret-ui-shadcn`) commonly need CSS-like `object-fit` behavior:

- `cover` for avatars and card media,
- `contain` for previews and attachments,
- `fill`/stretch only when explicitly desired.

Today, `SceneOp::Image` effectively behaves like `object-fit: fill` (stretch to the destination
rect). This causes visible parity gaps for Radix/shadcn components (avatars stretch) and pushes
each ecosystem crate to re-implement crop math ad-hoc, which is a high-risk “future rewrite”
pressure point.

Separately, Fret already has a stable fit vocabulary for viewport surfaces:

- `ViewportFit::{Stretch,Contain,Cover}` in `crates/fret-core/src/viewport.rs`

We should align image fit semantics with viewport fit semantics so that:

- still images,
- streaming video frames (ADR 0121/0126),
- and engine viewports (`RenderTargetId` + `SceneOp::ViewportSurface`)

share the same mental model and avoid divergent behavior.

Related ADRs:

- Resource handles and flush points: `docs/adr/0004-resource-handles.md`
- Streaming images and video surfaces: `docs/adr/0121-streaming-images-and-video-surfaces.md`
- Streaming update effects and metadata: `docs/adr/0126-streaming-image-update-effects-and-metadata-v1.md`
- Color/compositing contracts: `docs/adr/0040-color-management-and-compositing-contracts.md`

## Decision

### 1) Add a stable fit field to `SceneOp::Image` (v1)

Extend `SceneOp::Image` to include a fit mode, using a stable vocabulary aligned with
`ViewportFit`:

- `Stretch` (equivalent to CSS `object-fit: fill`)
- `Contain` (equivalent to CSS `object-fit: contain`)
- `Cover` (equivalent to CSS `object-fit: cover`)

Default behavior for v1:

- If the fit field is omitted at a higher-level API, it defaults to `Stretch` to preserve legacy
  behavior until callers opt in.

### 2) Fit applies only to `SceneOp::Image`, not `SceneOp::ImageRegion`

`SceneOp::ImageRegion` remains “caller-specified UV” and does not apply fit logic.

Rationale:

- Fit + explicit UV is ambiguous (does fit happen inside the UV sub-rect, or does the UV sub-rect
  override fit?).
- Keeping `ImageRegion` as the explicit escape hatch preserves predictability and avoids future
  breaking changes.

### 3) Rendering-only semantics (no intrinsic layout)

Fit affects only how pixels are sampled/mapped when drawing.

- Layout remains explicit and is driven by the destination rect chosen by the UI/layout engine.
- Streaming image size changes do not implicitly trigger relayout. Components that want aspect
  ratio constraints must express them explicitly (e.g. `aspect_ratio` wrappers).

## Semantics (normative)

Let:

- destination rect be `D` (logical pixels, in scene space),
- source image size be `(sw, sh)` (pixels),
- `D.w` and `D.h` be destination width/height (logical pixels).

### Stretch

Sample the full image over the full destination rect:

- Draw rect: `D`
- UV rect: full `UvRect::FULL`

### Contain

Preserve aspect ratio and letterbox/pillarbox inside `D`.

- Compute uniform scale `s = min(D.w / sw, D.h / sh)`
- Draw size: `(dw, dh) = (sw * s, sh * s)`
- Draw rect is centered in `D` with size `(dw, dh)`
- UV rect: full `UvRect::FULL`

### Cover

Preserve aspect ratio and crop to fully cover `D` (center-crop).

- Compute uniform scale `s = max(D.w / sw, D.h / sh)`
- Scaled size: `(cw, ch) = (sw * s, sh * s)`
- Compute the crop window in scaled coordinates:
  - crop width `D.w`, crop height `D.h`
  - center it in `(cw, ch)`
- Convert crop window to normalized UVs over the source image:
  - `u0 = ( (cw - D.w) / 2 ) / cw`
  - `v0 = ( (ch - D.h) / 2 ) / ch`
  - `u1 = 1 - u0`
  - `v1 = 1 - v0`
- Draw rect: `D`
- UV rect: `(u0, v0, u1, v1)`

Notes:

- v1 is center-crop only. Alignment/anchor points are out of scope (future extension).
- If `sw == 0` or `sh == 0` or `D.w <= 0` or `D.h <= 0`, the draw is a no-op.

## Precision, rounding, and pixel snapping (normative)

The fit math defined above operates in **logical pixels** (scene space) and produces:

- a destination draw rect in logical pixels (for `Contain`), and/or
- a normalized UV rect in `[0, 1]` (for `Cover`).

Rules:

1. Implementations must compute UVs in floating point and clamp to `[0, 1]`.
   - UVs must be monotonic (`u0 <= u1`, `v0 <= v1`) after clamping.
2. Implementations must avoid early rounding that changes semantics.
   - In particular, do not quantize `dw/dh` (contain) or UVs (cover) to integer pixels as part of
     the contract-level mapping. Any pixel-quantization belongs to the renderer’s vertex encoding
     and rasterization step.
3. Pixel snapping, if desired, is a renderer policy and must not change which pixels are sampled
   (i.e. must not alter UV crop semantics).

Rationale:

- Different backends (native vs web) have different rasterization/rounding behavior. Keeping the
  fit mapping continuous in logical space reduces drift and makes cross-backend conformance tests
  feasible.

## Consequences

- Ecosystem components can request `Cover`/`Contain` without re-implementing crop math.
- Video players using streaming `ImageId` updates gain correct fit behavior “for free”.
- Viewport surfaces and still images share a consistent fit vocabulary, reducing drift.

## Follow-up work (non-normative)

- Consider a shared `ObjectFit` type used by both viewports and images (instead of duplicating
  enums), if we want to expose CSS-aligned naming (`Fill/Contain/Cover`).
- Consider an optional image metadata query seam (`ImageService`) to support aspect-ratio-driven
  layouts in ecosystem components without app side channels.
- Consider an extended fit surface (v2) with alignment/anchor points (`Center`, `Top`, `TopLeft`,
  etc.) and optional sampling policy (`Nearest` vs `Linear`), but keep v1 minimal to avoid baking
  component policy into the core draw contract.
