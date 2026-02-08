# ADR 0089: Renderer Architecture v2 - Scene Compiler, Atlases, and Default Performance

Status: Accepted (scene compiler + caching landed; ongoing perf expansion)

## Context

Fret's goal is a general-purpose application UI framework that also scales to Unity-class editor UIs
(dense panels, trees, tables, toolbars, docking, multiple viewports, multi-window tear-off).

Fret already has several "hard-to-change" contracts that bias toward correctness and portability:

- The public display list (`fret-core::Scene`) is an ordered stream of `SceneOp` (ADR 0002).
- The renderer must preserve operation order across primitive kinds; batching is adjacency-preserving only (ADR 0009).
- Viewport surfaces (embedded engine render targets) and overlays require strict layering correctness (ADR 0007 / ADR 0011 / ADR 0015).
- The UI runtime is retained-mode and supports frame recording + subtree replay caching (ADR 0005 / ADR 0055).
- Text is atlas-backed with explicit quality targets (ADR 0006 / ADR 0029).
- Icons are SVG-first, semantic-keyed, and renderer-owned for caching/budgeting (ADR 0065).

Current implementation direction (summary, non-normative):

- `fret-ui` builds a `Scene` and can replay ranges from a previous frame on cache hits.
- `fret-render` encodes `SceneOp` into an internal ordered draw stream and issues wgpu draws.
- `Quad` is instanced; other primitives currently encode into compact vertex streams and typically still produce one draw per op.

That last point is the major scalability risk for editor-grade workloads: toolbars and lists tend to produce many
small textured quads (icons/glyphs) where CPU-side encoding and GPU-side draw/bind churn can dominate.

This ADR proposes a renderer architecture that:

1) keeps the public semantics stable (operation order is authoritative),
2) makes "default performance" a first-class requirement,
3) stays compatible with future wasm/mobile backends (wgpu/WebGPU),
4) keeps an explicit escape hatch for future semantic evolution if telemetry shows it is needed.

References:

- Display list contract: `docs/adr/0002-display-list.md`
- Ordering/batching contract: `docs/adr/0009-renderer-ordering-and-batching.md`
- Frame recording + replay: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- Text pipeline/atlas: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- Icons/SVG: `docs/adr/0065-icon-system-and-asset-packaging.md`
- Rounded clip semantics: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- Scene transform/clip composition: `docs/adr/0078-scene-transform-and-clip-composition.md`
- Zed/GPUI reference (renderer + batching patterns): `repo-ref/zed/crates/gpui/`
- Godot reference (editor UI via Canvas batching):
  - Cull/submit: `repo-ref/godot/servers/rendering/renderer_canvas_cull.cpp`
  - Command list + batching: `repo-ref/godot/servers/rendering/renderer_canvas_render.h`
  - Backend record/batch (RD): `repo-ref/godot/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp`

## Implementation Status (Current)

This ADR is accepted because the core “scene compiler” shape is implemented today in `fret-render`:

- A dedicated encoding pass produces an ordered internal draw stream (while preserving `Scene.ops` order).
- The encoder maintains explicit stacks (transform/opacity/scissor/clip) to match core contracts.
- Encoded results are cached by a stable key so identical frames can skip re-encoding.
- GPU uploads use a small ring buffer (frames-in-flight) to avoid writing into in-flight buffers.

However, the performance work described here is incremental: some parts (e.g. unifying all textured draws
into a single instanced Sprite2D path) may still be phased in as needed.

### Code map (as of current implementation)

- Scene encoding (compiler):
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/mod.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/state.rs`
  - `crates/fret-render-wgpu/src/renderer/types.rs` (`SceneEncoding`, `OrderedDraw`)
- Encoding cache key and reuse:
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (`SceneEncodingCacheKey`)
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (cache storage)
- Frames-in-flight buffers (ring buffers):
  - `crates/fret-render-wgpu/src/renderer/resources.rs` (`FRAMES_IN_FLIGHT = 3`)
- Atlas + budgets + perf snapshot (SVG path today):
  - `crates/fret-render-wgpu/src/renderer/config.rs` (`SvgPerfSnapshot`, budgets, clear-cache knobs)
  - Stress harness: `apps/fret-svg-atlas-stress/src/main.rs`

## Reference Notes: How Godot Handles Editor UI Rendering

Godot's editor UI is rendered via the same 2D/GUI "Canvas" pipeline used by in-game UI:

- UI nodes emit an ordered command stream (CanvasItem Command list).
- A cull/sort stage builds the final draw list (by z/layer/ysort) and submits it to the renderer.
- The render backend records commands and forms batches by breaking only on relevant state changes, while preserving order.
- Rects, ninepatches, and common primitives are heavily instanced (per-instance data + batch list).
- The renderer uses ring-buffered per-frame GPU storage (triple buffering) to avoid stalls.

Key takeaway for Fret: Godot achieves editor-grade UI density primarily via:

- (a) low-level instance-based submission,
- (b) stable "BatchKey" state splitting,
- (c) resource/texture caching,

not via global reordering that would violate an authoritative operation order contract.

## Goals

### Product goals

- Support Unity editor-grade UI density out of the box.
- Keep component authoring ergonomic, without forcing app developers to manually micro-optimize rendering.

### Technical goals (rendering/perf)

- Preserve strict compositing correctness (operation order remains authoritative).
- Make default performance observable and enforceable:
  - stable counters (draw calls, binds, bytes uploaded),
  - stable budgets (atlas memory, raster budgets),
  - reproducible stress harnesses.
- Reduce per-op overhead for the common hot primitives:
  - icons, images, text glyphs, viewport surfaces.
- Keep portability:
  - wgpu-first (Windows prioritized), but no design dead-ends for wasm/mobile.

## Non-Goals

- Replacing the retained UI runtime with a purely immediate-mode runtime.
- Introducing global cross-op reordering as a default optimization.
- Betting on platform-specific graphics APIs (DX12/Metal) as the primary route to performance.

## Decision

### 1) Treat the renderer as a "Scene Compiler"

We introduce an explicit internal stage:

`Scene (public contract)` -> `SceneCompiler` -> `CompiledScene` -> `Backend execution (wgpu)`

`SceneCompiler` responsibilities (internal, non-public):

- Normalize `SceneOp` into a small set of GPU-friendly primitive encodings.
- Apply adjacency-preserving run coalescing across all primitive kinds (not just quads).
- Drive resource preparation (atlas allocation, uploads, bind group updates) in a predictable order.
- Produce:
  - compact GPU buffers (instances/vertices),
  - a minimal sequence of "render runs" to execute in order,
  - per-frame `RenderStats`.

This formalizes what currently exists informally ("encoding") and makes it a stable place to:

- cache compiled results,
- test ordering/batching correctness,
- improve performance without touching `fret-ui`.

### 2) Converge on instance-driven 2D primitives

We define internal render IR families designed to cover most editor UI:

#### 2.1 Shape2D (SDF/analytic quads)

Instanced, GPU-evaluated shapes:

- rounded rect fills,
- borders,
- shadows/elevation (per ADR 0030 / ADR 0060 evolution).

#### 2.2 Sprite2D (textured quads)

Instanced textured quads, covering:

- `Image` / `ImageRegion`,
- `MaskImage` (alpha mask + tint),
- `SvgMaskIcon` / `SvgImage` (after rasterization),
- `Text` (after glyph expansion),
- `ViewportSurface` (engine render target as a sampled texture).

Key property:

- Most "textured UI things" become Sprite2D instances referencing a small number of texture pages/atlases.

#### 2.3 Mesh2D (colored triangles)

Paths (and future arbitrary meshes) remain mesh-like:

- `SceneOp::Path` stays a vertex stream (triangulated triangles).

### 3) Define a universal adjacency batching model (RenderRuns + BatchKey)

We keep ADR 0009: no cross-op reordering.

But we generalize batching beyond quads by introducing:

- `RenderRun`: a contiguous slice of one instance/vertex buffer executed by one pipeline variant.
- `BatchKey`: the minimal state needed to guarantee that coalescing adjacent items preserves output.

Two encoded items can coalesce iff:

1) They are adjacent in `SceneOp` order (or adjacent after expansion of the same op), and
2) Their `BatchKey` is identical, and
3) Coalescing does not change the semantics of state stacks (transform/opacity/clip), and
4) Their GPU execution path is stable for ordering within a run (same assumption already used for instanced quads).

#### 3.1 Sprite2D instance layout (draft)

Sprite2D is designed so that "one instance = one textured quad":

- `rect_px`: `[x, y, w, h]` in physical pixels
- `transform_rows`: 2x4 transform rows (or compact affine 2x3 + padding)
- `uv_rect`: `[u0, v0, u1, v1]`
- `color`: premultiplied RGBA (tint and/or alpha)
- `opacity`: optional separate alpha factor (or folded into color)
- `texture_page`: `u32` page index (bind-array path) OR encoded as a run-level bind key (non-bindless path)
- `kind`: sampled-as-RGBA vs sampled-as-mask (for mask/tinted icons vs color images)

This enables:

- icons, images, glyphs, and viewport surfaces to share the same draw mechanics,
- run-level or bindless texture selection.

#### 3.2 BatchKey definition (draft)

The BatchKey should be explicit and testable. A suggested structure:

- `pipeline_kind`: `Shape2D | Sprite2D | Mesh2D`
- `pipeline_variant`:
  - Sprite: `ColorSprite | MaskSprite | ViewportSprite`
  - Shape: `QuadSdf` (and future variants)
- `blend_mode`: premultiplied alpha vs other modes (if supported)
- `scissor_rect`: (x, y, w, h) in physical pixels
- `clip_mode`:
  - `ScissorOnly` (fast path)
  - `ShaderClip` (rounded/non-axis-aligned clip; shader uses clip buffer head)
- `uniform_slot`: index/offset of the uniform snapshot (including clip head/count)
- `texture_bind_key`:
  - non-bindless: a stable key describing the bound texture view + sampler (e.g. `TexturePageId`)
  - bindless: fixed; `texture_page` is per-instance and does not break batches
- `sampler_key`: filter/repeat/compare (should be small and stable)
- `color_space_key`: linear/srgb sampling expectation if the backend distinguishes
- `atlas_kind` (optional but useful for debugging): glyph_mask/glyph_color/svg_mask/svg_rgba/ui_image/viewport

BatchKey fields that frequently break batches should be minimized. This is why "few textures by default"
(atlas/page design) is a top-level goal.

### 4) Make "few textures" the default via texture pages and atlases

Draw-call/bind overhead is dominated by:

- number of distinct textures referenced, and
- how frequently bindings must change.

Decision: the renderer owns a page-based texture pool where most UI-visible raster content is expressed as:

- `TexturePageId` (atlas page or standalone texture page),
- `UvRect` (sub-rect in that page),
- sampling metadata (filter/repeat),
- optional color-space metadata.

Recommended "page kinds" (initial set):

- `GlyphMaskAtlas` (R8)
- `GlyphColorAtlas` (RGBA)
- `SvgMaskAtlas` (R8, page-based)
- `SvgRgbaPages` (RGBA, standalone pages or atlas if small)
- `UiImagePages` (RGBA, small images may be atlas-backed)
- `ViewportPages` (external engine textures, usually standalone)

Policy principle:

- Small, repeated assets should prefer atlas pages.
- Large, unique assets should be standalone pages (do not poison atlases).

### 5) Buffer and upload strategy: ring buffers and scratch reuse

To make performance stable (especially at 120Hz and debug builds), the compiler/execution pipeline must:

- avoid per-frame allocations in hot paths,
- avoid GPU stalls due to writing into in-flight buffers.

Decisions:

- Use ring-buffered GPU storage for frequently updated buffers (instances/vertices/uniforms).
  - Default: triple buffering (3 frames in flight), matching common engine practice (and Godot RD's approach).
- Use scratch `Vec` reuse for CPU-side staging (when mapping is not directly available).
- Cap per-frame buffer growth and surface explicit "dropped / overflow" stats in debug builds.

### 6) Caching layers and invalidation/revision keys

We retain the existing multi-layer caching model and make it explicit:

1) UI runtime cache: subtree replay caching of `SceneOp` ranges (ADR 0055).
2) Scene compiler cache: reuse `CompiledScene` when the authoritative scene input is identical.
3) Resource caches:
   - text blob cache + glyph atlas cache (ADR 0029),
   - SVG raster cache + mask atlas cache (ADR 0065),
   - texture page pool cache (image pages and reuse).

Compiler cache key should be stable and include at least:

- `scene_fingerprint` + `ops_len` (existing `SceneRecording::fingerprint()`),
- viewport size and scale factor,
- texture pool generation (invalidated when pages are evicted/reallocated),
- render target generation (viewport surfaces),
- relevant text/svg system revisions.

### 7) Observability is part of the renderer contract

The v2 renderer must surface stable per-frame metrics (per window):

- `scene_ops_len`
- expanded primitive counts (shape instances, sprite instances, mesh vertices)
- render runs: total and per `pipeline_variant`
- draw calls and bind-group switches
- bytes uploaded per buffer class (instances, vertices, uniforms/clips, textures)
- atlas stats (pages live, used %, evictions, budget)
- CPU time breakdown (compile, svg prepare, text prepare)

If we cannot measure it, we cannot "default optimize" it.

## Consequences

### Expected benefits

- Large reductions in draw calls for icon/text-heavy panels without changing the UI authoring model.
- Clear separation of responsibilities:
  - UI runtime focuses on invalidation and frame recording,
  - renderer focuses on compilation, atlases, and GPU execution.
- Cleaner long-term path to wasm/mobile:
  - fewer textures,
  - fewer state changes,
  - explicit budgets and fallbacks.

### Costs and risks

- Refactor cost: Sprite2D instancing and moving text/SVG into sprite instances touches hot paths.
- Correctness risk: batching rules must be tested heavily (clip/transform/opacity/viewport layering).
- GPU ordering assumptions: instanced draws are relied upon to preserve within-run ordering (already assumed for quads).

## Migration Plan (Implementation Strategy)

This plan is structured so that each phase improves performance and observability without requiring a "flag day".

### Phase 0: Lock metrics + stress harnesses (must happen first)

- Add `RenderStats` / perf snapshots and a debug output path.
- Add repeatable stress harnesses:
  - icon-heavy (SVG mask icons, different sizes),
  - text-heavy (many lines, selection highlights, mixed glyph types),
  - mixed UI: toolbars + tree/list virtualization + hover + viewport overlay.
- Add regression thresholds in CI for at least draw calls and bytes uploaded in a headless render test where possible.

### Phase 1: Universal adjacency coalescing (no semantic changes)

- Extend adjacency merging beyond quads:
  - merge adjacent image draws with identical state,
  - merge adjacent mask draws,
  - merge adjacent text draws (already partially grouped by kind, but extend to full adjacency).
- Remove avoidable per-frame allocations (scratch reuse for uniform packing and staging).

### Phase 2: Introduce Sprite2D instancing for images/svg/viewports

- Replace the current "6 vertices per image op" encoding with sprite instances.
- Keep existing texture binding strategy initially (bind group per texture page), but move to explicit page IDs.
- Ensure SVG rasters and icon masks prefer atlas pages with budgets/eviction.

### Phase 3: Expand Text to sprite instances (glyph quads become sprites)

- Convert text glyph quads into sprite instances referencing glyph atlases.
- Maintain two texture sources (mask vs color) via a pipeline variant or per-instance kind.
- Preserve shader-side quality controls from ADR 0029.

### Phase 4: Consolidate texture pages further (icon/image policies)

- Default icons to atlas-backed mask pages.
- Add an image policy that encourages small UI images to use atlas-backed pages.
- Introduce explicit budgets and eviction telemetry for each page kind.

### Phase 5 (Optional): Bindless-like sampling via binding arrays (capability-gated)

If the target device supports wgpu binding arrays:

- Bind a fixed-size array of texture views/samplers once.
- Encode `TexturePageId` as an index used per-instance.
- This removes most per-run bind churn and improves batching without changing order semantics.

Fallback:

- if binding arrays are unavailable (common on WebGPU/mobile), atlas/page design keeps texture count low enough
  to remain performant.

## Alternatives Considered (Semantic Changes)

### Alternative A: Make DrawOrder semantic and allow global sorting

Pros:

- Large batching potential (GPUI-like).

Cons:

- Conflicts with ADR 0082 and the "operation order is authoritative" mental model.
- High risk of breaking viewport/overlay layering invariants.
- Requires deep semantic redesign of overlays, clips, and compositing groups.

### Alternative B: Add explicit "BatchScope" / "CompositingGroup" semantics (opt-in reordering)

Pros:

- Opt-in path for extreme workloads (e.g. huge tables) without forcing global semantic complexity.

Cons:

- Requires a new semantics surface and conformance testing for alpha/overlap rules.

Recommendation:

- Defer until after v2 compiler + atlases land and we have telemetry showing we still need more.

## Open Questions

1) Formal guarantee for instanced ordering:
   - Should we codify that within a single instanced draw, instance order is treated as authoritative for compositing when instances overlap?
2) Clip/scissor strategy for maximal batching:
   - When (if ever) do we allow per-instance discard ("software scissor") to reduce run splits?
3) Texture pool and eviction:
   - Default budgets per class (glyph, svg mask, svg rgba, ui images, viewport).
4) Unified vs split pipelines:
   - Keep Shape2D and Sprite2D separate (simpler), or unify into one uber-pipeline (fewer pipeline switches, more shader complexity)?
5) Cross-platform constraints:
   - Which binding-array features can we rely on for future WebGPU/mobile targets, and what is the fallback story?

