# SVG / Path / Plot Rendering Workstream Notes

Status: active workstream notes (not a repo-wide roadmap). The authoritative roadmap lives in
`docs/roadmap.md`.

## Goal

Build a GPUI/Zed-aligned rendering foundation for:

- SVG icons/images (SVG-as-icon and SVG-as-image).
- Vector paths (lines/curves) as a general accelerated primitive.
- Higher-level “plot/implot-like” widgets built on top (data → paths + interactions).

## Non-goals (for now)

- Full GPU-native SVG path rendering (no SVG → GPU path commands directly).
- GPUI-style analytic coverage AA for curves (we rely on MSAA for now).
- A full plot component MVP before the rendering substrate is stable.

## Baseline (what fret does today)

### SVG (GPUI-aligned)

- CPU rasterization: `usvg` parses + `resvg/tiny-skia` renders into a CPU pixmap.
- Two productized upload paths:
  - **Alpha mask icon**: store `alpha` only (R8Unorm), tint on GPU (`SceneOp::SvgMaskIcon`).
  - **RGBA image**: store unpremultiplied RGBA (Rgba8UnormSrgb), premultiply in shader (`SceneOp::SvgImage`).
- Cache is inside the renderer (scene-driven prepare), not exposed as an external “svg cache object”.

### Path (implot substrate)

- CPU tessellation: `lyon` converts `PathCommand` into triangles (fill/stroke).
- GPU drawing:
  - When MSAA > 1: draw paths into an offscreen MSAA intermediate, then composite back to keep strict draw order.
  - When MSAA == 1: draw directly into the main pass (fast path).

## GPUI/Zed reference alignment (high level)

- SVG: CPU rasterize → GPU composite (matches GPUI).
- Icon path: alpha mask + tint on GPU (matches GPUI “monochrome sprite” concept).
- Paths: CPU-generated geometry; GPU ultimately still draws triangles (same fundamental primitive).

## Work items (incremental plan)

### Done

- SVG scene ops + UI primitives: `SvgMaskIcon` / `SvgImage`.
- SVG raster caching inside `fret-render` with byte-budget + LRU epoch.
- `SvgFit` modes: `Contain` (default), `Width`, `Stretch`.
- Path MSAA samples configurable, and MSAA==1 uses direct draw fast path.
- Path MSAA is capability-driven per format (auto fallback to supported sample counts).
- SVG alpha-mask **atlas pages** (reduce bind group / texture switching for many icons).
- SVG alpha-mask atlas uses an allocator (`etagere`) so entries can be deallocated and space reused (GPUI-style).

### Next

1. Atlas page lifecycle knobs (optional)
   - Pages are reusable internally, but not automatically “compacted” across pages (no moving allocations).
   - Keep explicit rebuild knobs (`Renderer::clear_svg_mask_atlas_cache()` / `clear_svg_raster_cache()`) as the primary operator control.
2. Capability-driven defaults
   - Decide default `path_msaa_samples` (compat-first vs quality-first).
3. Plot substrate (renderer stays generic)
   - Keep renderer responsible only for “draw paths efficiently”.
   - Build plot widget responsible for:
     - data → paths (downsampling, line joins, area fills, markers),
     - hit testing / hover / zoom & pan,
     - axes / ticks / labels (text).

## Key design stance: “Renderer accelerates paths; plot owns semantics”

The renderer should expose a small set of general primitives that map cleanly to GPU work:

- Rect quads (already exists).
- Images and alpha masks (already exists).
- Paths (already exists; triangles under the hood).
- Optional: clipped layers / offscreen surfaces (only if GPUI does it and we need it).

Plot/implot-like widgets should stay in UI/component crates and only emit those primitives.

## Resource lifecycle & reclamation (GPUI-inspired)

- Keep caches internal to `fret-render`:
  - SVG raster cache keyed by `(SvgId, target_box, smooth_scale, kind, fit)`.
  - Alpha-mask atlas pages registered as images; multiple icons share one `ImageId`.
- Reclamation:
  - Byte budget (`svg_raster_budget_bytes`) governs **standalone** rasters only (e.g. RGBA images, or alpha masks that can’t fit in the atlas).
  - Alpha-mask atlas pages use an allocator (free-rect packing), so removing entries can reclaim sub-rects for reuse.
  - Atlas pages are reclaimed only when explicitly cleared or when a page becomes empty.
  - Best-effort eviction: never evict standalone rasters used in the current frame.
  - Explicit knob: `Renderer::clear_svg_raster_cache()` drops all cached rasterizations without unregistering `SvgId`.

## Risks / pitfalls (what to watch)

1. Atlas sampling artifacts (bleeding)
   - Linear filtering can bleed across neighbors without padding.
   - MVP mitigates with padded + edge-extruded writes for each icon.
2. Budget vs correctness
   - If the current frame needs more than the budget, eviction must not break drawing.
   - Implementation must allow temporary overshoot (correctness first).
3. MSAA compatibility
   - Some backends/devices may not support the chosen MSAA sample count for a format.
   - Prefer capability-driven or conservative defaults; treat MSAA as a quality knob.
4. CPU SVG cost
   - Rasterization is CPU-heavy; too many unique sizes per frame will hurt.
   - Encourage consistent icon sizing and caching; avoid animating target size every frame.
5. Fragmentation
   - Allocator reuse prevents per-page “append-only” growth, but there is no cross-page compaction.
   - If churn causes page count to grow, use explicit rebuild (`clear_svg_mask_atlas_cache`) or consider an eviction policy.

## Validation checklist

- SVG icon alpha-mask:
  - Same SVG tinted with multiple colors looks consistent (ignores original SVG fills).
  - No visible bleeding when many icons are packed in the atlas.
- SVG image RGBA:
  - Semi-transparent SVG has no “dark/white fringe” (premul/unpremul correctness).
- Path:
  - With MSAA off, edges are visibly jaggier (expected).
  - With MSAA on, edges improve and ordering stays correct with other draws.

## Local validation demo

- Run: `cargo run -p fret-svg-atlas-stress`
- Headless (CI-friendly-ish): `cargo run -p fret-svg-atlas-stress -- --headless --frames 600 --budget-kb 1024`
  - End-to-end-ish (wait GPU): `cargo run -p fret-svg-atlas-stress -- --headless --frames 600 --budget-kb 1024 --wait-gpu`
  - Fragmentation/churn probe: `cargo run -p fret-svg-atlas-stress -- --headless --frames 600 --budget-kb 1024 --churn --churn-every 180 --churn-drop 64`
- Controls:
  - `Space`: toggle phase (A/B)
  - `T`: toggle auto phase flip
  - `F`: cycle `SvgFit` mode
  - `B`: cycle `svg_raster_budget_bytes` presets
  - `H`: print help to stdout
