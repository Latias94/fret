> Archived: keep for history; prefer ADRs + `docs/roadmap.md` for active decisions.

# MVP: Vector Graphics Roadmap (SVG + Paths)

This document is intentionally standalone to avoid interfering with existing design docs used by others.

## Goals

- SVG icons: support "monochrome icon + tint" efficiently (GPUI-aligned).
- SVG images: support RGBA SVG as a normal image.
- Vector paths: render fill/stroke paths with good quality, predictable ordering, and reasonable performance.
- Keep the renderer responsible for "generic accelerated drawing"; keep higher-level components (e.g. plots) responsible for "data → geometry + interaction policy".

## Non-goals (for this MVP)

- Full SVG scene graph rendering on GPU (we follow GPUI: CPU rasterization for SVG).
- A complete plot/component library like ImPlot (this will be layered on top later).
- Automatic LRU eviction without a safe lifetime model (explicit removal only for now).

## GPUI Reference (why we chose these paths)

- SVG is CPU rasterized (usvg + resvg + tiny-skia), then uploaded to GPU:
  - Monochrome icon: alpha mask + tint (sprite-like).
  - RGBA image: upload as a normal image texture.
- Path is CPU tessellated (lyon) into triangles, then rendered with a dedicated AA path pipeline; GPUI also uses an offscreen intermediate and compositing to preserve draw order.

We mirror this split in Fret.

## Current Status (Implemented)

- SVG alpha-mask + tint
  - CPU rasterization: `crates/fret-render-wgpu/src/svg.rs`
  - UI primitive: `crates/fret-ui/src/element.rs` (`SvgIconProps`)
  - Scene op: `crates/fret-core/src/scene.rs` (`SceneOp::MaskImage`)
  - Upload helpers: `crates/fret-render-wgpu/src/svg.rs`
- SVG RGBA image upload
  - CPU rasterization: `SvgRenderer::render_rgba*` in `crates/fret-render-wgpu/src/svg.rs`
  - GPU upload: `upload_rgba_image` in `crates/fret-render-wgpu/src/svg.rs`
- Path rendering + offscreen MSAA composite
  - Scene op: `crates/fret-core/src/scene.rs` (`SceneOp::Path`)
  - Renderer intermediate + composite: `crates/fret-render-wgpu/src/renderer/mod.rs`
- Caching + explicit reclamation (GPUI-aligned)
  - Global cache type: `crates/fret-render-wgpu/src/svg_cache.rs` (`SvgImageCache`)
  - Eviction API: `clear/remove_alpha_mask/remove_rgba`
- Demo wiring (cache stored as App global resource)
  - Runner hook: `crates/fret-launch/src/runner/mod.rs` (`gpu_frame_prepare`)
  - Demo usage: `apps/fret-demo/src/bin/components_gallery.rs`

## Design Boundary (Renderer vs Plot/Components)

Renderer responsibilities:

- Define low-level scene ops and execute them efficiently (`SceneOp::*`).
- Provide accelerated primitives: images, text, rectangles, paths, mask images.
- Maintain correct draw order, including when offscreen passes are involved.

Plot/component responsibilities:

- Convert domain data to vector paths (or batches of paths).
- Decide styling (color/width), sampling/downsampling strategies, and interaction (hover/selection/zoom).
- Manage its own local caches (e.g. resampling), but rely on renderer for GPU caches via stable IDs.

This separation matches GPUI's implementation split: `PathBuilder`/renderer pipeline vs higher-level UI elements.

## Risks / Watch-outs

- CPU cost for SVG rasterization:
  - Resvg/tiny-skia runs on CPU; large SVGs or many distinct sizes can cause spikes.
  - Mitigation: cache by `(svg hash, target size, smooth scale factor, kind)`; avoid re-rasterization in a frame.
- Memory growth due to caching:
  - Each cached entry holds a GPU texture; without LRU/refcount, memory can grow unbounded.
  - Mitigation (MVP): explicit `clear` / `remove_*` aligned with GPUI's `atlas.remove(...)`.
- Offscreen intermediate overhead:
  - Path MSAA intermediate adds extra passes and textures; can be bandwidth-heavy.
  - Mitigation: only allocate when needed; reuse across frames; keep sample count small.
- Ordering correctness:
  - Introducing offscreen passes must not reorder scene ops.
  - We treat "runs of path ops" carefully and composite back into the main pass in order.
- Quality vs performance tradeoffs:
  - Smooth SVG scale factor improves icon quality but increases CPU cost.
  - Path AA quality depends on shader technique and MSAA; keep it configurable later.

## Next Steps (Suggested Execution Order)

1. Ergonomics: add a higher-level `SvgIcon` / `SvgImage` element that hides manual preparation, while keeping the same renderer primitives.
2. Path API completeness: extend `StrokeStyle` (cap/join/miter), add optional dashes, and expose fill rule explicitly (already present, but expand usage).
3. Clipping + transforms: ensure consistent clip behavior for mask/path/image operations.
4. Plot MVP (after above foundations):
   - Start with a line plot (single series) → multi-series → axes/labels → zoom/pan → selection.
   - Keep it a component crate; use `PathCommand` as the output target and rely on existing path/mask primitives.

