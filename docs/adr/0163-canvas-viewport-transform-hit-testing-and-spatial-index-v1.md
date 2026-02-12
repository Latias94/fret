# ADR 0163: Canvas Viewport, Transforms, Hit Testing, and Spatial Index (Toolkit v1)

- Status: Proposed
- Date: 2026-01-17
- Scope: `ecosystem/fret-canvas` view/hit-test/spatial toolkit contracts used by editor-grade retained canvases.
- Related:
  - ADR 0078 (Scene transform and clip composition)
  - ADR 0082 (Render transform hit testing)
  - ADR 0128 (Canvas widgets and interactive surfaces)
  - ADR 0141 (Declarative Canvas element and painter)
  - ADR 0144 (Canvas pan/zoom input mapping v1)
  - ADR 0152 (Kurbo geometry backend for canvas hit testing)
  - ADR 0161 (Canvas cache policy and hosted resource caches)
  - ADR 0162 (fret-canvas modules and feature strategy)

## Context

Fret has multiple “canvas-like” editor surfaces that draw large 2D scenes and require consistent interaction:

- node graphs (`ecosystem/fret-node`) with ports/wires and heavy hit testing,
- plots and charts (`ecosystem/fret-plot`, `ecosystem/fret-chart`) with overlays and zoomable regions,
- future canvases (timelines, gizmos, CAD-like editors, large data grids).

These canvases repeatedly implement the same infrastructure concerns:

- coordinate spaces (screen/layout/canvas/world) and bidirectional mapping,
- view transforms (pan/zoom) and consistent hit slop across zoom/DPI,
- clip conventions and “overlay escape” patterns,
- coarse candidate lookup (spatial indices) and refined hit testing (geometry),
- progressive work scheduling to avoid frame spikes (budgeted preparation).

ADR 0128 defines canvas terminology and high-level guidance. This ADR locks a **minimal, reusable toolkit contract**
in `ecosystem/fret-canvas` so ecosystem canvases converge instead of drifting.

## Goals

1. Provide a shared, portable canvas toolkit surface for:
   - viewport and transform mapping,
   - hit testing helpers (including zoom/DPI-aware slop),
   - coarse spatial indexing for large scenes.
2. Keep the contract **policy-light**:
   - no tool modes, no gesture maps, no domain rules.
3. Support both:
   - retained canvases that own their state and caches, and
   - declarative `Canvas` as a runtime mechanism (ADR 0141).
4. Make performance tuning actionable by enabling consistent instrumentation (debug/diagnostics).

## Non-goals

- A single universal “Canvas widget” data model for all editors.
- A dependency on a specific renderer backend or platform API.
- Exposing heavy backend types (`kurbo`, `rstar`) in public contracts.
- Perfect global optimization in v1; we standardize building blocks first.

## Decision

### 1) Canonical coordinate mapping helpers live in `fret-canvas::view`

`ecosystem/fret-canvas` provides a small, reusable mapping surface for 2D canvases.

Locked terminology aligns with ADR 0128:

- **Screen / window space**: window-local logical pixels used by `Event` coordinates.
- **Layout space**: the widget’s untransformed `bounds`.
- **Canvas space**: a canvas-defined “world” coordinate system used to author large content.

Contract shape (names are normative, exact fields are flexible):

- `CanvasViewport2D` (or equivalent):
  - stores the widget bounds (layout rect),
  - stores the canvas view transform (pan/zoom) as a `Transform2D`,
  - provides `screen_to_canvas`, `canvas_to_screen`, and rect variants.

Guidance:

- When the entire surface transforms together, use `Widget::render_transform` (ADR 0082) and keep
  mapping helpers for overlay anchoring and hit slop conversion.
- When only a sub-region transforms (e.g. plot region, not axes), keep explicit mapping per region.

### 2) Clip and overlay conventions remain explicit, but helpers may exist

Clipping semantics are owned by the scene/runtime (ADR 0078/0087/0063). Canvas toolkits:

- must not bypass core clip semantics,
- may provide helpers to compute clip rects for a mapped canvas region,
- should encourage overlay escape patterns (ADR 0128 + ADR 0064).

### 3) Hit testing is a two-stage pipeline (locked)

All editor-grade canvases should follow a two-stage approach:

1. **Coarse candidate lookup** (spatial index; fast; may over-approximate).
2. **Refined hit test** (geometry; exact enough to feel stable).

`fret-canvas` provides:

- zoom/DPI-aware slop helpers:
  - `hit_slop_canvas_units(dpi_scale_factor, zoom, slop_screen_px) -> f32`,
  - `stroke_width_canvas_units(...)` helpers for constant-pixel strokes (existing `scale` module).
- geometry helpers remain feature-gated (ADR 0152):
  - default lightweight implementations,
  - optional `kurbo` backend for correctness/reference.

### 4) Spatial index contract is standardized in `fret-canvas::spatial`

The spatial index is a coarse accelerator for:

- querying items in a viewport rect (culling),
- querying candidates near a pointer position (hit testing),
- incremental updates during drags (move/update/remove).

Locked properties:

- portable types (`fret_core::{Point,Rect}`),
- queries may return duplicates; callers may sort/dedup as needed,
- stable call sites via a backend wrapper.

Backend strategy (aligned with ADR 0162):

- default: uniform grid with backrefs (portable, simple),
- optional: `rstar`-powered backend behind `fret-canvas/rstar`,
- backend types are not exposed in the public contract.

### 5) Progressive work and budgets are first-class (recommended contract)

To keep interaction “smooth by default”, canvases should be able to bound expensive work:

- path preparation/tessellation,
- text shaping/preparation,
- large-index rebuilds,
- SVG parsing/registration.

This ADR recommends a small portable budget helper in `fret-canvas` (module name TBD):

- `FrameBudget` / `WorkBudget`:
  - tracks per-frame limits for “work units” (entries prepared, items refined, etc.),
  - supports `try_consume(n)` patterns for incremental work loops.

Budgeting is a toolkit mechanism; policy (numbers, priorities, fallback UI) stays in the consumer.

## Consequences

Pros:

- Ecosystem canvases converge on consistent coordinate mapping and hit-testing patterns.
- Large-scene interaction scales better (spatial index becomes a reusable substrate).
- Makes future “smoothness” work (budgeted preparation, progressive rendering) easier to apply broadly.

Cons / risks:

- If the toolkit becomes too large, `fret-canvas` risks becoming a “misc utils” bucket (ADR 0162).
- Multiple existing canvases will require migration to take full advantage of the shared toolkit.

## Implementation Plan (phased, recommended)

1. Document and stabilize the v1 toolkit surface:
   - `fret_canvas::view` mapping helpers and naming,
   - `fret_canvas::spatial` contract (backend wrapper + backrefs).
2. Adopt and harden in the highest-pressure canvas:
   - `ecosystem/fret-node` (node/port/edge hit testing).
3. Expand to additional canvases:
   - `ecosystem/fret-plot` and `ecosystem/fret-chart` for overlay anchoring and consistent slop.
4. Add instrumentation:
   - per-canvas counters for candidate counts, refine counts, and budget saturation events,
   - integrate with diagnostics bundles (ADR 0159) without coupling to policy.

## Evidence / existing building blocks

- Spatial indexing substrate:
  - `ecosystem/fret-canvas/src/spatial.rs` (grid + backrefs + backend wrapper)
  - `ecosystem/fret-canvas/src/spatial_rstar.rs` (optional backend)
  - Bench examples: `ecosystem/fret-canvas/examples/{spatial_index_bench,node_graph_spatial_bench}.rs`
- Node-graph coarse indexing usage:
  - `ecosystem/fret-node/src/ui/canvas/spatial.rs`
- View transform and input mapping building blocks:
  - `ecosystem/fret-canvas/src/view/*`
  - `ecosystem/fret-ui` `Widget::render_transform` contract (ADR 0082)

## Open Questions

1. Should `fret-canvas` standardize a shared “hit-test slop” key builder and naming scheme so call
   sites don’t reimplement hashing and tuning?
2. Do we want a dedicated diagnostics registry for canvas “interaction perf” (candidates/refines)
   analogous to the cache stats registry added in ADR 0161?
3. Should the budget helper integrate with renderer capability tiers (ADR 0116/0121) or remain
   entirely app-owned in v1?

