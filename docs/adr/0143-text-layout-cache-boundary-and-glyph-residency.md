# ADR 0143: Text Layout Cache Boundary and Glyph Residency

- Status: Proposed
- Date: 2026-01-13

## Context

Fret’s stable UI boundary for text is already locked:

- `TextService::prepare(TextInput, TextConstraints) -> (TextBlobId, TextMetrics)` (ADR 0006),
- geometry queries on prepared blobs (ADR 0045 / ADR 0046),
- atlas-backed glyph rendering (ADR 0029),
- deterministic truncation semantics (ADR 0059),
- v2 shaping direction via Parley + attributed spans (ADR 0142).

The current renderer implementation (as of the Parley v2 workstream) caches:

1) A **layout/shaping result** (caret stops, line metrics, glyph positions), and
2) The **glyph atlas placement** (UVs / atlas page) in the same cached output (`TextShape`).

This coupling causes long-term architectural friction:

- **UI retention drives renderer memory**: if a `TextBlobId` is kept alive (even when offscreen),
  its glyphs must remain resident to keep `uv/atlas_page` valid.
- **Eviction semantics are constrained**: eviction must be conservative (pinned by blobs), which
  makes tight budgets difficult under churn and virtualized surfaces.
- **Scene encoding cache can bypass glyph preparation**: if text glyph residency depends on
  `encode_text` running, a cached encoding can skip “ensure glyphs exist” work.
- **Prepare latency is inflated**: `prepare(...)` must rasterize and allocate glyphs eagerly
  even when the blob may never be rendered.

To be future-proof for editor-grade workloads (virtualized, high volume, high DPI),
the renderer needs an explicit, stable boundary between:

- “text layout is stable and cacheable”, and
- “glyph atlas residency is a renderer resource with budgets and eviction”.

## Goals

1) Make glyph atlas eviction/budgets **independent** of `TextBlobId` retention.
2) Ensure `TextBlobId` remains valid and geometry queries remain correct even when glyphs are evicted.
3) Keep deterministic layout semantics (wrap/ellipsis + caret/hit-test) derived from the same layout.
4) Enable a future “streaming glyph residency” model suitable for virtualization and backpressure.
5) Keep the stable cross-crate boundary: UI does not shape or rasterize.

## Decision

### 1) Split caches: layout cache vs glyph residency cache

The renderer text system is split into two conceptual subsystems:

- **Layout cache**: produces a stable `TextLayout` (or equivalent) from `TextInput + constraints`.
  This output contains glyph *instances* (glyph key + position) and the full geometry query state
  (lines, caret stops, selection mapping), but **must not embed atlas UVs**.

- **Glyph residency cache**: maps `GlyphKey -> AtlasEntry` and owns allocation, uploads, and eviction
  under a configured budget (pages/bytes).

### 2) Make glyph residency a frame-driven “ensure” step

Glyph residency must be ensured in a step that runs **even when scene encoding is cached**.

The renderer calls a new hook (exact naming TBD):

- `TextSystem::prepare_for_scene(scene, scale_factor, ...)`

This method:

- walks the scene for `SceneOp::Text` usages,
- gathers required glyph keys for visible blobs,
- ensures those glyphs are present in the atlas (alloc + upload scheduling),
- performs bounded eviction based on recent usage.

`render_scene` then calls `text_system.flush_uploads(queue)` after the ensure step.

### 3) Rendering resolves atlas UVs at draw planning time

`encode_text` (or an internal “text draw planner”) resolves each `GlyphInstance`:

- `GlyphInstance { key, rect, paint_span, kind }`
- `GlyphCache::lookup(key) -> AtlasEntry { page, uv_rect }`

This yields GPU vertices with UVs, without storing UVs inside the layout cache.

### 4) Canonical glyph keys

The renderer uses a canonical `GlyphKey` for rasterization cache keys:

- font identity (blob/id + index),
- glyph id,
- size,
- subpixel bins,
- raster content kind (mask/color),
- future: hinting/variation/font-features if required.

This aligns with the existing `GlyphRasterKey` direction, but the long-term goal is to remove
backend-specific variants once v2 fully converges on Parley (ADR 0142).

## Consequences

- `prepare(...)` can become cheaper (layout-only), but first render of a blob may trigger uploads.
- Renderer complexity increases: one more “ensure” stage and explicit residency tracking.
- Scene encoding cache remains valuable, but must not be relied on for glyph residency.
- Atlas eviction can be tighter and more predictable, enabling better behavior under virtualization.

## Alternatives Considered

1) Keep UVs in `TextShape` and pin glyphs by blob lifetime.
   - Simpler, but prevents future budget correctness and virtualization-friendly behavior.

2) Add an explicit “prune text cache” API called by apps.
   - Helps memory, but still couples correctness to caller discipline and does not solve encoding-cache bypass.

3) Increase atlas budgets and avoid eviction.
   - Works for demos, but fails for long-running/editor workloads and mobile/wasm constraints.

## Implementation Plan (incremental)

1) Introduce an internal `TextLayout` output that carries `GlyphInstance` keys and geometry state
   (no UVs), while keeping the public boundary unchanged.
2) Add `TextSystem::prepare_for_scene(...)` and move glyph atlas insertion to this step.
3) Update `Renderer::render_scene` ordering so glyph residency runs before upload flush and does not
   depend on scene encoding recomputation.
4) Implement an eviction policy based on “last used frame/epoch”, with a clear “in-flight safety”
   story (do not evict resources referenced by submitted command buffers).
5) Remove legacy coupling (drop per-blob pinning once frame residency is correct) and converge
   fully on Parley for shaping/rasterization keys (ADR 0142).

