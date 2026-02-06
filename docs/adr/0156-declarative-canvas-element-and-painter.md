# ADR 0156: Declarative Canvas Element and Painter (Resource-Hosted Custom Draw)

Status: Proposed
Scope: `crates/fret-ui` runtime substrate (mechanism) + ecosystem layering guidance.

## Context

Fret needs a "canvas-like escape hatch" for:

- short-term custom drawing without defining a full custom component, and
- building complex, interactive canvas surfaces (node graphs, plots, charts, editors) while keeping
  overlays, clipping, transforms, and input routing aligned with the runtime contracts.

Fret already has the underlying rendering substrate:

- retained/declarative UI emits portable `SceneOp`s (`docs/adr/0002-display-list.md`),
- `render_transform` maps paint + hit-testing + pointer coordinates consistently (`docs/adr/0083-render-transform-hit-testing.md`),
- clipping and effect semantics are explicit and must not be bypassed (`docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`,
  `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`),
- overlays are explicit roots and are placed via the overlay placement contract (`docs/adr/0064-overlay-placement-contract.md`).

What is missing is a **declarative** low-level "Canvas" element analogous to the GPUI/Zed escape hatch:

- GPUI provides `canvas(prepaint, paint)` and uses it for "short term custom drawing"
  (`repo-ref/zed/crates/gpui/src/elements/canvas.rs`).

Fret has an additional constraint that GPUI does not: most non-trivial draw ops require **prepared
resource handles** with explicit release (`TextBlobId`, `PathId`, `SvgId`), and the UI layer must
release these deterministically via `UiServices` (see `docs/adr/0004-resource-handles.md`).

If we add a naive declarative canvas that emits `SceneOp::{Text,Path,SvgImage}` directly, component
authors will be forced to implement their own caches and resource teardown logic. This is not
compatible with a declarative-only authoring direction (`docs/declarative-only-migration.md`).

## Goals

1. Provide a declarative `Canvas` element for low-level custom drawing, aligned with the existing
   layout/clip/effect semantics (no bypass).
2. Make resource lifecycle **safe by default**: canvas authors should not need to manually track
   and release `TextBlobId`/`PathId`/`SvgId` for typical usage.
3. Keep interaction policy out of the runtime: input maps, tool modes, snapping, etc. remain
   ecosystem/app-owned (ADR 0137 direction).
4. Support both "one-off custom draw" and "canvas-like widget" composition patterns.
5. Preserve portability (no backend/platform deps).

## Non-goals

- Introduce a second rendering system; `SceneOp` remains the one draw-list substrate.
- Mandate a single shared "Canvas" data model for charts/graphs/editors.
- Encode gesture maps, tool modes, or domain-specific transactions into `crates/fret-ui`.
- Provide a stable exported retained-widget API as the primary authoring surface.

## Decision

### 1) Add a declarative `Canvas` element to the runtime substrate

`crates/fret-ui` adds a new declarative element kind:

- `ElementKind::Canvas(CanvasProps)`

The element participates in layout as a rectangular region and paints inside the normal runtime
paint pipeline, subject to:

- `Overflow::Clip` clipping conventions (ADR 0088 / ADR 0063),
- effect layers and opacity stacks (ADR 0119),
- ancestor `render_transform` behavior (ADR 0083).

The canvas element itself is paint-only; pointer/keyboard routing remains explicit and is composed
using existing primitives such as `PointerRegion`, `Pressable`, and `RenderTransform`.

Locked details:

- The `Canvas` element is a **leaf** (no children). Overlay composition is done explicitly by
  wrapping the canvas in layout/stack primitives (e.g. `Stack`) or by using overlay roots (ADR 0011
  / ADR 0064).

### 2) Provide a `CanvasPainter` API that hosts prepared resources

The runtime provides a paint callback interface that receives a mechanism-oriented painter:

- `CanvasPainter` owns element-local caches for prepared resources (text blobs, paths, SVGs).
- The painter exposes "draw" helpers that:
  - prepare resources via `UiServices`,
  - cache by a stable key,
  - emit the corresponding `SceneOp`,
  - and release all cached resources when the element is removed (via `Widget::cleanup_resources`).

This makes "custom draw" usable in a declarative world without forcing every canvas author to
re-implement cache + teardown logic.

Locked details:

- The first implementation scope is limited to hosted caching for:
  - `TextBlobId` (text),
  - `PathId` (vector paths),
  - `SvgId` (SVG images).
  Additional resource kinds (e.g. images/materials) are explicitly deferred.

### 3) Caching semantics: deterministic, keyed, and scale-aware

The painter caches resources by a key derived from:

- a caller-provided stable key (`u64`), and
- the relevant preparation constraints (at minimum `TextConstraints.scale_factor` /
  `PathConstraints.scale_factor` / analogous SVG constraints).

This ensures that DPI / zoom-dependent resource preparation does not silently reuse stale assets.

The runtime provides (or recommends) a small `KeyBuilder` helper for composing keys from IDs,
revisions, and small scalars.

Locked details:

- The cache key input is **required** (no "magic hashing" of arbitrary inputs in v1).
- The runtime must always incorporate the relevant scale factor bits into the effective cache key
  (even when the caller key is stable) to prevent stale reuse under DPI/zoom changes.
- Any “shared text caching” convenience API (hashing caller inputs) must be **explicitly opted into** by the call site
  and must not be used implicitly by the keyed `text/text_with_blob` helpers. This avoids accidental high-entropy cache
  pollution in tight paint loops.
- The runtime does **not** infer zoom from transforms. Callers that want higher-resolution prepared
  resources under a canvas zoom must pass `scale_factor = dpi * zoom` (e.g. via
  `fret_canvas::scale::effective_scale_factor`).

### 4) Layering: runtime mechanism, ecosystem policy

This ADR locks a layering split aligned with ADR 0137:

- `crates/fret-ui` owns only the mechanism (Canvas element + painter + resource hosting).
- `ecosystem/fret-canvas` remains policy-light substrate helpers (pan/zoom math, scale helpers,
  generic drag vocabulary, lightweight caches).
- Optional policy packages (e.g. standard pan/zoom gesture controller, inertia curves, shortcut
  gates) belong in an ecosystem "kit" layer (e.g. `ecosystem/fret-canvas-kit` or `fret-ui-kit`),
  not in `crates/fret-ui`.

### 5) Migration direction: declarative-first canvases

Complex canvases should be implemented as declarative compositions:

- input routing via `PointerRegion`/action hooks,
- view transforms via `RenderTransform` (pan/zoom) where appropriate,
- painting via `Canvas` + hosted painter,
- overlays via overlay roots and anchored placement.

Retained widgets may remain as transitional glue where necessary, but new shared retained widget
surfaces should not be introduced as public ecosystem authoring primitives (consistent with
`docs/declarative-only-migration.md`).

### 6) Paint callback registration uses element-local state (not props)

Fret's declarative `ElementInstance` records are `Clone`/`Debug` and are stored in a per-frame
registry. This makes storing arbitrary closures in props impractical.

Therefore, the canvas paint callback is registered via element-local state (action hook pattern,
ADR 0074 direction):

- A `Canvas` element establishes the element ID scope.
- During rendering, user code registers a paint handler for the scoped element via `ElementContext`.
- During paint, the host widget resolves and runs the last registered handler for that element.

The runtime must clear per-frame paint handlers at the start of each render pass for the element so
stale handlers cannot persist when user code stops registering them.

## API Sketch (Non-normative)

This is illustrative, not a locked Rust signature.

### Short-term custom draw (Divider-like)

- A layout container sets size/overflow.
- `Canvas` draws using `CanvasPainter`, relying on runtime-managed caches.

### Infinite canvas (pan/zoom)

- A pointer region owns interaction state and updates a `PanZoom2D` model.
- A render-transform wrapper applies the view transform to the canvas subtree.
- The canvas draws world content in canvas-space coordinates.

### Engine viewport

Engine viewports are not implemented via `Canvas`; they continue to use `ViewportSurface`
(ADR 0007) plus explicit `ViewportInput` forwarding (ADR 0147 / ADR 0025 direction).

## Consequences

Pros:

- Declarative custom draw becomes ergonomic and safe in the presence of explicit resource handles.
- Canvas content naturally composes with runtime clip/transform/effect semantics.
- Ecosystem canvases can converge on a common authoring pattern without adopting a shared data model.

Cons:

- The runtime surface area grows (new element + painter contract).
- The painter must be carefully scoped to avoid becoming a "policy sink".

## Open Questions

1. Should the runtime provide a dedicated prepaint/bounds hook (GPUI-style), or should authors rely
   on the paint callback's `bounds` plus existing `last_bounds_for_element` queries?
2. How should non-uniform transforms (e.g. scale_x != scale_y) influence "effective scale factor"
   helpers for text/path preparation, if we later choose to infer scale from transforms?
3. Do we need a "hosted cache budget" policy (max entries, LRU) for long-lived canvases, or should
   that remain app/component-owned in v1?

## References

- Canvas guidance and layering direction: `docs/adr/0137-canvas-widgets-and-interactive-surfaces.md`
- Render transforms (paint + hit test + event coords): `docs/adr/0083-render-transform-hit-testing.md`
- Display list substrate: `docs/adr/0002-display-list.md`
- Resource handle lifecycle: `docs/adr/0004-resource-handles.md`
- Rounded clipping and overflow conventions: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`,
  `docs/adr/0088-overflow-and-clipping-conventions.md`
- Effect layers: `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Overlay placement: `docs/adr/0064-overlay-placement-contract.md`
- GPUI/Zed canvas element reference: `repo-ref/zed/crates/gpui/src/elements/canvas.rs`
