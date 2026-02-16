---
title: XYFlow (React Flow) gap analysis (Fret canvas + node graph)
status: active
date: 2026-02-13
scope: ecosystem/fret-canvas, ecosystem/fret-node, ecosystem/fret-ui-ai (workflow wrappers)
---

# XYFlow (React Flow) gap analysis (Fret canvas + node graph)

This document answers a practical question for the AI Elements workflow ports:

> AI Elements uses `@xyflow/react` (`ReactFlow`). What do we still *lack* in Fret to support the
> same user-facing workflow experiences?

It is **not** a goal to recreate the React/DOM API surface. Instead we map the behavior/semantics
to existing Fret layers and note what is missing.

## Version stamp (upstream reference)

Local snapshot (optional repo-ref checkout):

- `repo-ref/xyflow`

Pinned commit:

- `7813d07345250d38b6fb292cbf4da58cc9f6a5df` (commit date `2026-02-09`)

## Upstream behavior anchors (files worth reading)

Viewport + input filtering:

- `repo-ref/xyflow/packages/react/src/container/ZoomPane/index.tsx`
- `repo-ref/xyflow/packages/system/src/xypanzoom/XYPanZoom.ts`
- `repo-ref/xyflow/packages/system/src/xypanzoom/filter.ts`
- `repo-ref/xyflow/packages/system/src/xypanzoom/eventhandler.ts`

Selection-on-drag (marquee) semantics:

- `repo-ref/xyflow/packages/react/src/container/Pane/index.tsx`

Delete-key semantics:

- `repo-ref/xyflow/packages/react/src/hooks/useGlobalKeyHandler.ts`

## Where Fret already matches (or exceeds) XYFlow

### 1) Viewport model + fit view

XYFlow:

- `viewport: { x, y, zoom }` and helpers like `setViewport`, `fitView`, `zoomTo`.

Fret equivalents:

- `ecosystem/fret-canvas/src/view/pan_zoom.rs`: `PanZoom2D` model (pan + zoom).
- `ecosystem/fret-canvas/src/view/constraints.rs`: `fit_view_to_canvas_rect(...)`.
- `ecosystem/fret-node/src/ui/viewport_helper.rs`: `NodeGraphViewportHelper` (set viewport, fit view,
  set center), which is intentionally close to XYFlow’s `useReactFlow()` helper semantics.

### 2) Background patterns (lines/dots/cross)

XYFlow:

- `Background` component supports multiple variants and sizes.

Fret equivalents:

- `ecosystem/fret-node/src/ui/style.rs`: `NodeGraphBackgroundStyle` / `NodeGraphBackgroundPattern`
  explicitly reference XYFlow `BackgroundVariant` knobs.

### 3) Selection semantics (headless)

XYFlow:

- Drag-to-select rectangle (`selectionOnDrag`) plus modifier-dependent add/toggle/replace behavior.

Fret equivalents:

- `ecosystem/fret-canvas/src/interaction/selection.rs`: headless helpers for click-vs-box selection,
  normalized rects, and default modifier-to-mode mapping.

### 4) Editor-grade node graph widget (turnkey)

If the product needs a real workflow editor (node/edge editing, hit-testing, selection, commands),
Fret already has a much richer surface than XYFlow:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`: retained `NodeGraphCanvas` widget with:
  - wheel pan + ctrl/cmd wheel zoom,
  - marquee selection,
  - connect-drag wiring,
  - command integration (delete selection, frame all/selection, copy/paste, etc.),
  - minimap + controls overlays in its style model.

## What is still missing / mismatched (gap list)

This list is ordered by “what we would need to build an XYFlow-like *component* experience” (not
by what the retained `fret-node` widget already has).

### Gap A — Declarative "world layer" for nodes as element subtrees (partially closed)

XYFlow’s core affordance is: nodes are DOM elements, edges are SVG, both live in a transformed
world that pans/zooms together.

In Fret today (2026-02-12):

- A minimal reusable world-layer composition helper exists:
  - `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_surface_panel`)
  - Supports both:
    - `CanvasWorldScaleMode::ScaleWithZoom` (XYFlow-like)
    - `CanvasWorldScaleMode::SemanticZoom` (editor-like)
- A UI Gallery spike exists with a diag gate:
  - Page: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`
  - Gate: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json`

What is still missing (v1 ergonomics gaps):

- **Interaction glue**: a world-layer-only solution still needs app-level policies for:
  - node dragging (beyond the minimal recipe; snaplines / snap-to-grid / auto-pan),
  - connection-drag (handles, loose/strict targeting),
  - selection model updates (click vs marquee).
- **Current bounds latency caveat**: the v0 world layer derives the transform from a
  `LayoutQueryRegion` using `layout_query_bounds(...)` (last-frame bounds). This can produce a
  one-frame mismatch on resize/layout changes. See `docs/workstreams/canvas-world-layer-v1.md`.

Notes:

- A minimal bounds store seam exists (frame-lagged) for fit-view + selection queries:
  - `ecosystem/fret-canvas/src/ui/world_layer.rs` (`CanvasWorldBoundsStore`, `canvas_world_bounds_item`)
- A minimal node dragging spike exists (still app-owned):
  - `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`
- A minimal connect-drag spike exists (still app-owned):
  - Handles + preview: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (`ui-ai-cwl-node-a-source-handle`, `ui-ai-cwl-node-b-target-handle`)
  - Deterministic commit helper: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (`ui-ai-cwl-commit-connection`)
  - Gate: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (assert `ui-ai-cwl-connection-committed`)

Recommendation:

- Track remaining work as `docs/workstreams/canvas-world-layer-v1.md` M2 (bounds + selection seams).
- For editor-grade workflows **today**, prefer `fret-node::NodeGraphCanvas` as the interaction engine
  and use `fret-ui-ai` workflow wrappers as chrome.

### Gap B — ReactFlow-like input filter knobs (`noWheel` / `noPan` / `.nokey`)

XYFlow includes fine-grained input filtering:

- Ignore wheel or pan when the event target is inside a `noWheelClassName` / `noPanClassName` subtree.
- Allow marquee selection “above” nodes by using pointer events capture, unless the down event is
  within a `.nokey` subtree (background-only selection-on-drag).

In Fret:

- Overlay elements can naturally “eat” pointer events by being above the canvas, but there is no
  standard recipe for “canvas is interactive unless event occurs inside an exempt subtree”.

Recommendation:

- Use the explicit `fret-canvas/ui` exemption + marquee recipes (hit-test based, not class-name based):
  - `ecosystem/fret-canvas/src/ui/input_exempt.rs`: `canvas_input_exempt_region` (`.nowheel` / `.nopan` equivalents).
  - `ecosystem/fret-canvas/src/ui/pan_zoom.rs`: `editor_pan_zoom_canvas_surface_panel_with_marquee_selection`
    (selection-on-drag / marquee overlay).
  - For **world-layer nodes as element subtrees**, prefer:
    - `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_surface_panel_with_marquee_selection`)
    - Rationale: marquee chrome must render above node subtrees (canvas-paint marquee would sit behind).
  - For XYFlow-style “background-only” selection-on-drag, use:
    - `ecosystem/fret-canvas/src/ui/pan_zoom.rs`: `CanvasMarqueeSelectionProps::start_filter`
    - A practical implementation is a bounds-store-based filter using `CanvasWorldBoundsStore`
      (see the UI Gallery spike + diag gate).

### Gap F — World-layer node bounds → viewport helpers (fit view) (partially closed)

XYFlow’s `fitView` works because the system has access to measured DOM bounds for nodes.

In Fret’s declarative world layer today, we want a first-class seam to collect per-node bounds
without forcing apps to invent ad-hoc measurement registries.

Current state (2026-02-12):

- A minimal bounds store + item wrapper exists:
  - `ecosystem/fret-canvas/src/ui/world_layer.rs`:
    - `CanvasWorldBoundsStore`
    - `canvas_world_bounds_item(...)`
- A minimal fit-view helper exists:
  - `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_fit_view_to_keys`)
- The UI Gallery spike uses this to show a live union rect (proof of wiring):
  - `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`

Target outcome:

- Keep the seam small and robust:
  - node subtrees can publish stable canvas-space bounds (fit-view and selection queries),
  - element IDs are preserved for screen-space queries (overlay anchoring via `visual_bounds_for_element`),
  - avoid per-pan/zoom thrash by keeping the stored values zoom-invariant in `ScaleWithZoom` mode.

Still missing:

- (v1) Selection-on-drag integration is now accounted for:
  - marquee chrome above nodes via `canvas_world_surface_panel_with_marquee_selection`,
  - background-only start via `CanvasMarqueeSelectionProps::start_filter`.

Notes:

- Bounds remain frame-lagged due to `LayoutQueryRegion` / element bounds cache semantics.

Workstream anchor:

- `docs/workstreams/canvas-world-layer-v1.md` (M2)

### Gap C — Dashed strokes for paths (edge temporary)

AI Elements’ `edge.tsx` temporary edge uses `strokeDasharray: "5, 5"`.

In Fret today:

- Declarative `CanvasPainter::path` renders a tessellated stroke but does not expose dash patterns.

Status: Closed (2026-02-12)

Implemented (polyline-level approximation, renderer unchanged):

- Geometry helper: `ecosystem/fret-canvas/src/wires.rs` (`dash_polyline_segments`,
  `cubic_bezier_polyline_points`)
- Applied in AI Elements workflow chrome: `ecosystem/fret-ui-ai/src/elements/workflow/edge.rs`
  (`WorkflowEdgeTemporary`)

Notes:

- This matches the upstream outcome for the temporary edge (`strokeDasharray: "5, 5"`) by emitting
  a set of independent stroked line segments along a flattened Bezier polyline.
- If/when the renderer grows native dash support, this should be migrated down to avoid per-segment
  path overhead and to better match corner joins/caps.

### Gap D — Edge markers (arrowheads) and richer routing primitives (component layer)

XYFlow supports marker end caps and multiple edge types (bezier, step, straight) and makes it easy
to supply a `markerEnd`.

In Fret today:

- `WorkflowEdge*` and `WorkflowConnection` are intentionally minimal canvas renderers.
- `fret-node` has much more advanced routing math, but it is not packaged as a general “edge kit”.
- A minimal arrowhead primitive exists and is used for AI Elements parity:
  - `ecosystem/fret-canvas/src/wires.rs` (`arrowhead_triangle`)
  - `ecosystem/fret-ui-ai/src/elements/workflow/edge.rs` (`WorkflowEdgeAnimated`)

Recommendation:

- Extract a small “edge rendering kit” out of `fret-node` (math + marker helpers) into `fret-canvas`
  (or a sibling) so both `fret-node` and `fret-ui-ai` workflow wrappers can share it.

### Gap E — A public "workflow controller" surface for `WorkflowCanvas`

XYFlow exposes helpers (`useReactFlow`) for `zoomIn/zoomOut/fitView` that pair naturally with the
Controls UI.

In Fret today:

- `WorkflowControls` exists as chrome, but it does not ship with a canonical controller type.

Recommendation:

- Provide an optional controller in `fret-ui-ai` that is model-only (no engine) and wires:
  - `zoom_in/zoom_out` (mutate `Model<PanZoom2D>`),
  - `fit_view_to_canvas_rect` when an app provides a target rect (seam).

## Practical guidance for app authors (today)

Pick one of two paths:

1) **Editor-grade workflow editor**
   - Use `fret-node::NodeGraphCanvas` as the engine.
   - Use `fret-ui-ai` workflow wrappers (`WorkflowPanel/Toolbar/Controls`) as overlay chrome if
     desired.
   - Skin `NodeGraphStyle` to match shadcn/AI Elements tokens (future work).

2) **Custom lightweight workflow surface**
   - Use `WorkflowCanvas` + `fret-canvas/ui` tool router for pan/zoom and tools.
   - Render nodes/edges in the canvas paint pass.
   - Avoid “DOM-like nodes as element subtrees” until a dedicated world-layer substrate exists.
