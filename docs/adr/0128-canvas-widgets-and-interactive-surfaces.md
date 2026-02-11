# ADR 0128: Canvas Widgets and Interactive Surfaces (2D, Plots, Node Graphs)

Status: Proposed
Scope: UI infrastructure contracts and guidance (portable). Domain policy remains ecosystem/app-owned.

## Context

Multiple ecosystem crates implement “canvas-like” retained widgets today:

- `fret-node`: `NodeGraphCanvas` (pan/zoom, large-scene drawing, spatial hit testing).
- `fret-plot`: `PlotCanvas` (plot regions + axes + overlays, cursor readouts, box zoom, queries).
- `fret-chart`: `ChartCanvas` (ImPlot-like interactions, axis bands, engine-driven marks).
- `fret-plot3d`: `Plot3dCanvas` (viewport surface + input forwarding).

These widgets repeatedly need to solve the same infrastructure questions:

- coordinate spaces and transforms (pan/zoom, data→px mapping),
- input mapping consistency under transforms,
- clip/hit-testing rules and capture behavior,
- overlay placement over transformed content,
- resource lifecycle and caching (paths/text blobs),
- semantics/a11y expectations for “viewport-like” surfaces.

Fret already has the underlying mechanisms:

- retained widgets can emit arbitrary `SceneOp`s during `paint`.
- `Widget::render_transform` maps paint + hit-testing + event coordinates consistently (ADR 0082).
- engine viewports are represented explicitly (`SceneOp::ViewportSurface`, ADR 0007) and input can be
  forwarded via data-only effects (ADR 0025).

What is missing is a single, explicit “canvas” contract that prevents drift across ecosystem
canvases and avoids future rewrites.

## Goals

1. Define what “canvas” means in Fret (without introducing a second rendering system).
2. Provide stable coordinate/transform terminology and recommended patterns.
3. Keep policy out of `fret-ui` (ADR 0027 / ADR 0066): the framework provides mechanisms and
   guardrails; ecosystem/app crates provide interaction policy (input maps, tools, rules).
4. Ensure canvas widgets remain compatible with:
   - overlays and multi-root composition (ADR 0011 / ADR 0064),
   - command routing and capture/focus (ADR 0020),
   - multi-window and viewport embedding (ADR 0017 / ADR 0007 / ADR 0025),
   - performance work (paint caching constraints under transforms) (ADR 0082 / ADR 0055).

## Non-goals

- A single universal “Canvas API” that all charts/graphs must use.
- Forcing charts/plots/node graphs into one shared data model.
- A DOM-like layering model inside canvases.

## Decision

### 1) “Canvas” is a pattern over existing retained widget + scene primitives

Fret does not introduce a new rendering surface for “canvas”.

Instead, a **canvas widget** is a retained widget that:

- participates in layout as a rectangular region (`bounds`),
- draws its content via `SceneOp`s in `paint`,
- optionally uses `render_transform` to apply a view transform (pan/zoom) to its subtree,
- handles pointer/keyboard events and may capture input per ADR 0020.

### 2) Two canonical classes of canvas surfaces

Fret recognizes two common classes:

1. **2D UI-drawn canvases** (node graphs, 2D plots, charts):
   - draw using quads/paths/text under a well-defined coordinate mapping.
2. **Engine viewport canvases**:
   - draw engine content via `SceneOp::ViewportSurface` (ADR 0007),
   - forward input via `Effect::ViewportInput` (ADR 0025).

Both classes should present similar UX expectations (focus, capture, semantics role) even though
their rendering sources differ.

### 3) Coordinate spaces and terminology (locked)

To avoid ambiguity, the following terms are used consistently:

- **Window space**: window-local logical pixels (the coordinate system used by `Event` positions).
- **Layout space**: the untransformed `bounds` space for a widget/node (layout-authoritative).
- **Render space**: the visual result after applying composed `render_transform` matrices.
- **Canvas space**: a widget-defined coordinate system used to author large content (often
  “world space” for node graphs); typically corresponds to layout space *before* applying
  `render_transform` if `render_transform` is used for view transforms.
- **Data space**: domain-specific coordinates (plot data units, time/value, etc.).

Guidance:

- Prefer implementing **view transforms** (pan/zoom) via `render_transform` (ADR 0082) when the
  entire canvas content should transform together.
- For plot/chart widgets where axes/chrome should not transform with the plot region, use explicit
  data→px mapping per region (as `PlotCanvas` does today) instead of forcing a single transform for
  the entire widget.

### 4) Overlays over canvas content (locked guidance)

Canvas widgets often need overlays (tooltips, readouts, context menus) that should remain
screen-space while anchoring to transformed content.

Fret supports two compatible approaches:

1. **Window overlays (preferred when the overlay must escape clipping)**:
   - Use multi-root overlays (ADR 0011) and anchored placement (ADR 0064).
   - Anchor using window-space geometry queries that include render transforms (ADR 0082).
2. **In-canvas overlays (acceptable for self-contained UX)**:
   - Render overlay UI within the canvas widget but keep sizes in screen pixels by scaling widths,
     radii, and stroke widths by `1/zoom` (node-graph style).

The framework does not mandate which approach each canvas uses, but it requires that overlays do
not introduce a new platform-specific window system.

### 5) Input, focus, capture, and command routing (locked)

Canvas widgets must follow ADR 0020:

- pointer capture dominates hit-testing,
- focus is window-local and overlay-aware,
- command routing is scope-aware (`Widget → Window → App`).

Guidance:

- Canvas widgets should request focus on pointer down when they intend to receive keyboard input
  (plot hotkeys, node graph shortcuts).
- Canvas widgets should release capture on cancel/escape paths.

### 6) Semantics (a11y) expectations (recommended)

Canvas widgets should expose at least a stable semantics role and label:

- `SemanticsRole::Viewport` for plot/graph/viewport surfaces is recommended.
- Labels should be user-visible nouns (“Plot”, “Node Graph”, “Viewport”) rather than debug ids.

Full internal semantics for canvas contents (nodes, axes ticks, handles) is a future extension and
should be capability- and performance-aware (ADR 0033).

### 7) Resource lifecycle and caching (locked guidance)

Canvas widgets commonly allocate renderer resources (paths/text blobs/images).

Requirements:

- Release renderer resource handles in `cleanup_resources`.
- Avoid per-frame churn by caching derived resources keyed by stable revisions:
  - model revision(s),
  - scale factor where relevant,
  - view transform parameters when they affect measurement.

Notes:

- When using `render_transform`, be aware that paint caching may be disabled or constrained (ADR 0082 / ADR 0055). Canvases should treat caching as “best effort” and still be correct without it.

### 8) What belongs in framework vs ecosystem

Framework-owned (portable mechanisms):

- retained widget contract (`paint/event/command/cleanup`),
- `render_transform` semantics (ADR 0082),
- overlay infrastructure (ADR 0011 / ADR 0064),
- viewport surface + input forwarding (ADR 0007 / ADR 0025).

Ecosystem/app-owned (policy and domain logic):

- input maps (modifier policies, mouse/keyboard gestures),
- spatial indices and hit-testing heuristics for domain primitives,
- data models (graph/plot/chart state) and undo/redo policy (ADR 0127).

### 9) Industry alignment (examples, non-normative)

Other UI stacks typically provide a “canvas-like” escape hatch with similar boundaries:

- Flutter: `CustomPaint` (draw callback) + gestures; overlays are separate widgets layered above.
- Jetpack Compose: `Canvas` composable; coordinate transforms + input handled in user code.
- SwiftUI: `Canvas` view; overlays are separate views; transforms affect drawing and hit testing.
- ImGui node editors: draw-list canvas with “suspend/resume” to render overlays in screen space.

Fret’s design is aligned:

- `SceneOp` is the draw-list equivalent.
- `render_transform` is the standard mechanism for “pan/zoom without breaking input”.
- overlays remain explicit roots rather than ad-hoc z-index.

Zed/GPUI reference (non-normative):

- GPUI provides a lightweight declarative `Canvas` element implemented as a pair of callbacks:
  `prepaint(bounds) -> T` and `paint(bounds, T)` (`repo-ref/zed/crates/gpui/src/elements/canvas.rs`).
  This is used for “short term custom drawing” without defining a full custom widget type.
- GPUI’s canvas paint runs inside the normal style paint pipeline (via `Style::paint`), ensuring
  that clipping/background/border radius behavior stays consistent with other elements. A future
  `fret-canvas` declarative canvas should follow the same principle and must not bypass core clip
  and effect semantics (ADR 0087 / ADR 0063 / ADR 0117).

Implication for Fret:

- Fret can support the same need without inventing a separate drawing system by providing an
  optional declarative canvas element (e.g. in `ecosystem/fret-canvas`) that bridges into `SceneOp`
  emission, while keeping retained widgets as the primary mechanism for complex canvases.

### 10) Typical canvas use cases (examples, non-normative)

These use cases are intentionally broad; they motivate the guardrails above and help validate that
Fret’s “canvas is a widget pattern” boundary is sufficient.

- **Infinite design canvas (Figma/Miro-like)**:
  - many shapes/text layers in an infinite plane, pan/zoom, selection/multi-select, alignment
    guides, snapping, screen-space handles, and anchored tooltips.
  - Requires clear separation between content transform (canvas space) and screen-space UI
    overlays (handles, guides, tooltips).
- **Pixel tools / emulator framebuffer**:
  - display and interact with a dense pixel grid (nearest-neighbor scaling, optional grid overlay),
    update at high frequency, pick/paint/select pixels.
  - Requires an efficient raster update path and a clear sampling policy.
- **Maps/GIS/tiled worlds**:
  - streaming tiles + overlays + picking; strong caching requirements and incremental updates.
- **Timeline / curve editor**:
  - large scrollable/zoomable 1D/2D coordinate spaces, drag handles, selection ranges, snapping,
    and constant-pixel stroke widths.
- **Waveform / spectrum editors**:
  - large-data progressive rendering (LOD), selection, cursor readouts.
- **Diagram/flow editors**:
  - node/edge editing, hit testing, routing, and overlay affordances (context menus/searchers).
- **CAD / technical drawing**:
  - precision transforms, stable hit-testing heuristics, and explicit overlay handles/constraints.

### 11) Boundary and extensibility (what Fret should provide vs what users extend)

This ADR intentionally does **not** mandate a single reusable `Canvas` type. Instead, it locks a
small set of framework capabilities and extension patterns so ecosystem canvases can converge
without becoming coupled to one data model.

#### a) What the framework should provide (capabilities)

Portable capabilities that canvas authors should be able to rely on:

- **Custom draw**: a retained widget can emit arbitrary `SceneOp`s in `paint`.
- **View transforms with correct input**: `Widget::render_transform` applies to paint + hit-testing
  + event coordinates (ADR 0082).
- **Clipping and hit-testing controls**: clip rect + optional rounded clip behavior, and opt-in
  hit-test transparency controls (retained widget contract).
- **Explicit overlays**: multi-root composition and deterministic overlay ordering (ADR 0011),
  plus anchored placement solver (ADR 0064).
- **Geometry queries for anchoring**: ability to anchor overlays to what the user sees (visual
  bounds that include render transforms), not just layout bounds (ADR 0082).
- **Continuous interaction scheduling**: `RequestAnimationFrame` / timers for hover and drags
  (ADR 0034), without forcing a global “always animate” mode.
- **Viewport surfaces** (for engine/simulator outputs): `SceneOp::ViewportSurface` + data-only
  input forwarding (`Effect::ViewportInput`) (ADR 0007 / ADR 0025).
- **Observation/invalidation**: model/global change propagation that makes “undo changes many
  panels” reliable (ADR 0051), and supports canvas redraw without bespoke glue.
- **Undo hooks, not policy**: begin/update/commit/cancel semantics should be expressible in app
  code without requiring `fret-ui` to own a history stack (ADR 0127).

Non-portable escape hatches (backend-only) may exist, but canvas widgets must remain correct without
them.

#### b) How users extend (recommended patterns)

Ecosystem/app crates can implement new canvases by following these patterns:

- Implement a retained widget that owns:
  - domain models (via `Model<T>`),
  - derived caches (spatial indices, prepared text/path resources),
  - interaction state (hover, capture, drag phases).
- Prefer explicit **view transforms** (pan/zoom) via `render_transform` when the entire content
  should transform together. Use region-local mapping when only a subset should transform (plots).
- Keep overlays explicit:
  - window overlays when the overlay must escape clipping or be shared across panels/roots,
  - in-canvas overlays when a self-contained UX is acceptable and constant-pixel sizing can be
    maintained under zoom.
- Ensure resource lifecycle is deterministic (`cleanup_resources`) and caches are keyed by stable
  revisions (model rev, scale factor, view transform params).

#### c) Raster/pixel surfaces (guidance; avoid inventing a new subsystem)

Raster/pixel-editor and emulator-framebuffer surfaces should be modeled using existing primitives:

- Prefer representing pixel buffers as **images/textures** that the renderer can draw, with the UI
  providing the interaction layer (selection, tools, overlays).
- For high-frequency updates, use (or add) a streaming update path with explicit budgets and
  backpressure rather than allocating a new image id every frame (see renderer streaming ADRs).
- Sampling policy (nearest-neighbor vs linear) is a renderer concern. If needed, add an explicit
  sampling hint to the relevant scene ops behind a capability gate, rather than embedding sampling
  hacks in each canvas widget.

## Consequences

Pros:

- Ecosystem canvases converge on shared terminology and stable patterns.
- Future performance and overlay work can target one contract rather than per-canvas rewrites.
- Keeps `fret-ui` mechanism-only while still supporting editor-grade custom surfaces.

Cons:

- No single shared `Canvas` type is mandated; some duplication may remain until common helpers are
  factored into an ecosystem crate/module.

## Implementation Notes (Non-normative)

Existing examples to keep aligned:

- Node graph pan/zoom via `render_transform`: `ecosystem/fret-node/src/ui/canvas/widget.rs`
- Plot regions + axis chrome without global transform: `ecosystem/fret-plot/src/retained/canvas/mod.rs`
- Engine viewport mapping + input forwarding: `ecosystem/fret-plot3d/src/retained.rs`
- Chart axis bands + engine-driven marks: `ecosystem/fret-chart/src/retained/canvas.rs`

Crate placement (recommended):

- Keep `crates/fret-ui` mechanism-only: do not add “canvas policy” (pan/zoom gestures, snapping,
  tool modes) into the UI runtime.
- Factor reusable, policy-adjacent canvas helpers into **ecosystem**, not core:

  Option A (no new crate, lowest friction):

  - Add small reusable helpers under `ecosystem/fret-canvas` (policy-light substrate) used by multiple canvases:
    - pan/zoom math helpers (`ecosystem/fret-canvas/src/view.rs`, `ecosystem/fret-canvas/src/scale.rs`),
    - constant-pixel stroke helpers (`ecosystem/fret-canvas/src/scale.rs`),
    - common drag phase state (`ecosystem/fret-canvas/src/drag.rs`) aligned with ADR 0127,
    - optional declarative wrappers (`ecosystem/fret-canvas/src/declarative/*`).
  - Pros: keeps canvas substrate reusable without pulling in UI kit policy/recipes.
  - Cons: still requires discipline to avoid growing it into a second UI runtime.

  Option B (recommended long-term; preferred):

  - Use a small dedicated crate `ecosystem/fret-canvas` for shared canvas substrate:
    - depends on `fret-core` + `fret-runtime` + `fret-ui` (no platform/backend deps),
    - contains only reusable value types + pure helpers (no shadcn design system),
    - can expose an optional `headless`/`declarative` feature when useful.
  - Pros: keeps layering clean; canvases can reuse substrate helpers without depending on UI kit
    policies/recipes; easier to version/scope.
  - Cons: adds one more crate; requires intentional API design to avoid premature over-abstraction.

Potential follow-up (optional):

- Add a small `fret-canvas` feature-gated “UI integration” module (e.g. `fret-canvas/ui`) with
  reusable canvas wiring helpers and value types:
  - pan/zoom transform helpers,
  - constant-pixel stroke utilities,
  - common drag phase state (`Begin/Update/Commit/Cancel`) aligned with ADR 0127.

## References

- RenderTransform semantics: `docs/adr/0082-render-transform-hit-testing.md`
- Overlays and multi-root composition: `docs/adr/0011-overlays-and-multi-root.md`
- Overlay placement: `docs/adr/0064-overlay-placement-contract.md`
- Focus and command routing: `docs/adr/0020-focus-and-command-routing.md`
- Viewport surfaces: `docs/adr/0007-viewport-surfaces.md`
- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`
- Multi-window and DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Accessibility bridge: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Paint caching (context): `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- Undo/redo boundary (hooks): `docs/adr/0127-undo-redo-infrastructure-boundary.md`
- Streaming surfaces (raster/video): `docs/adr/0119-streaming-images-and-video-surfaces.md`
- Streaming budgets/backpressure: `docs/adr/0121-streaming-upload-budgets-and-backpressure-v1.md`
