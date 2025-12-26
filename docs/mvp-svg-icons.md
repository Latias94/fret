# MVP: SVG Icons (Alpha Mask + Tint)

Status: **Draft / In Progress**

This document is intentionally standalone to avoid interfering with other in-flight docs.

## Goal

Enable **SVG icon rendering** in Fret with a design aligned with Zed/GPUI:

- Parse + rasterize SVG into an **alpha mask** (CPU).
- Upload alpha mask as a GPU texture.
- Draw the icon by sampling the mask and multiplying by a **tint color** (GPU).

This provides a high-quality and theme-friendly icon path without requiring a full SVG feature
surface (filters, gradients, nested masks, text shaping, etc.).

## Non-goals (for this MVP)

- Full SVG feature coverage (filters/gradients/complex clip-path/masks).
- Converting arbitrary SVG to `PathCommand` (vector-native rendering).
- Plot/ImPlot-style components (data → path strategies and interactions live above the renderer).
- A new “Triangles” primitive exposed to UI code (renderer stays triangle-based internally).
- New offscreen composition beyond what we already do; we only adopt it where GPUI already does.

## Reference: Zed/GPUI evidence

GPUI’s layering is the key precedent for this MVP:

- Paint phase inserts a high-level `Path` primitive into the scene (not triangles, not data):
  - `repo-ref/zed/crates/gpui/src/window.rs:3018` (`paint_path`)
- Scene models `Path` as a high-level primitive:
  - `repo-ref/zed/crates/gpui/src/scene.rs:199` (`Primitive::Path`)
- Renderer consumes `PrimitiveBatch::Paths` and uses an intermediate render target for path batches:
  - `repo-ref/zed/crates/gpui/src/platform/blade/blade_renderer.rs:706`
- SVG icons render as an **alpha mask**:
  - `repo-ref/zed/crates/gpui/src/svg_renderer.rs:95` (`render_alpha_mask`)

## Why alpha-mask + tint?

Pros:

- Theme-friendly (tint at draw time).
- Keeps renderer generic (just “draw a mask with a color”).
- Avoids committing to full SVG parsing semantics in our public API.
- Matches GPUI’s proven approach for icons.

Trade-offs:

- Icons are rasterized (scale-dependent). Quality is still good for icon sizes if we rasterize at
  appropriate scale and cache by size/scale-factor.
- Complex SVG features are not supported unless `resvg/usvg` covers them (and we choose to enable).

## Proposed architecture

### New scene primitive

Add a new `SceneOp` variant:

- `MaskImage { rect, image: ImageId, uv: UvRect, color: Color, opacity: f32 }`

Semantics:

- `image` points to a texture containing **coverage in the red channel** (`R8Unorm`).
- In the fragment shader:
  - `coverage = textureSample(mask, uv).r`
  - output premultiplied color: `vec4(color.rgb * coverage, color.a * coverage)`

### SVG rasterization (CPU)

Introduce `fret-render::SvgRenderer` (CPU-only):

- Input: SVG bytes and a target box/scale factor.
- Output: `SvgAlphaMask { size_px, alpha_bytes }`
- Implementation: `usvg` parse + `resvg` render to `tiny-skia::Pixmap`, then extract alpha channel.

This keeps SVG parsing out of the UI layer and out of the `fret-core` contract surface.

### Upload to GPU

Upload the mask as `wgpu::TextureFormat::R8Unorm` and register it as an `ImageId`.

Important detail: WebGPU requires `bytes_per_row` alignment. We must pad rows when uploading.

**Helper**: `fret-render` provides `upload_alpha_mask(...) -> UploadedAlphaMask` which handles row
padding for `R8Unorm` uploads.

### UI usage

Expose a small retained widget:

- `fret-ui::primitives::MaskImage`

So UI code can render icons with:

- `MaskImage::new(mask_image_id).tint(color).opacity(…).with_uv(…)`

### Quick usage sketch

1. Rasterize SVG bytes into an alpha mask (CPU).
2. Upload alpha mask into an `R8Unorm` texture and register it as an `ImageId`.
3. Draw with `MaskImage` and a tint color.

Pseudo-code:

```rust
let svg = fret_render::SvgRenderer::new();
let mask = svg.render_alpha_mask(svg_bytes, (16, 16))?;
let uploaded = fret_render::upload_alpha_mask(&device, &queue, &mask);
let image_id = renderer.register_image(fret_render::ImageDescriptor {
    view: uploaded.view.clone(),
    size: uploaded.size_px,
    format: wgpu::TextureFormat::R8Unorm,
    color_space: fret_render::ImageColorSpace::Linear,
});

// In UI paint:
// MaskImage::new(image_id).tint(Color::WHITE)
```

## Roadmap

### ICON-0: Mask primitive (renderer + UI)

- [ ] Add `SceneOp::MaskImage`
- [ ] Add renderer mask pipeline (same vertex format as text; texture bind group like images)
- [ ] Add `fret-ui::primitives::MaskImage`

### ICON-1: SVG alpha mask rasterization

- [ ] Add `fret-render::SvgRenderer::render_alpha_mask`
- [ ] Add unit test for alpha-mask generation (CPU-only)

### ICON-2: Caching + integration

- [ ] Add cache keyed by `(svg_hash, size_box, scale_factor)` producing `ImageId`
- [ ] Add optional “smooth scale factor” policy (similar to GPUI’s `SMOOTH_SVG_SCALE_FACTOR`)

## Risks / gotchas

- **Upload alignment**: `bytes_per_row` must be padded to `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`.
- **Color management**: mask textures are linear; tint should be handled in the same color space as
  other premultiplied draws.
- **API creep**: avoid baking SVG-specific concepts into `fret-core`; keep SVG parsing behind
  `fret-render`.
- **Performance**: rasterization must be cached (especially under DPI changes).

## Notes on egui / egui_plot

Egui’s rendering model ultimately emits meshes (triangles) and relies heavily on batching/caching.
For plot-like widgets, egui_plot performs data reduction and interaction in the widget layer, then
emits geometry for the renderer. This aligns with the same separation principle:

- renderer = generic drawing acceleration
- plot/widget = data-to-geometry strategy + interaction
