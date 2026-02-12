---
title: XYFlow (React Flow) gap analysis (Fret canvas + node graph)
status: active
date: 2026-02-12
scope: ecosystem/fret-canvas, ecosystem/fret-node, ecosystem/fret-ui-ai (workflow wrappers)
---

# XYFlow (React Flow) gap analysis (Fret canvas + node graph)

This document answers a practical question for the AI Elements workflow ports:

> AI Elements uses `@xyflow/react` (`ReactFlow`). What do we still *lack* in Fret to support the
> same user-facing workflow experiences?

It is **not** a goal to recreate the React/DOM API surface. Instead we map the behavior/semantics
to existing Fret layers and note what is missing.

## Version stamp (upstream reference)

Local snapshot (developer machine asset, not part of this repo):

- `F:\SourceCodes\Rust\fret\repo-ref\xyflow`

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

### Gap A — Declarative "world layer" for nodes/edges as element subtrees

XYFlow’s core affordance is: nodes are DOM elements, edges are SVG, both live in a transformed
world that pans/zooms together.

In Fret today:

- `WorkflowCanvas` (`ecosystem/fret-ui-ai/src/elements/workflow/canvas.rs`) is a **host** surface
  built on `fret-canvas/ui` pan/zoom + an overlay slot.
- `WorkflowNode` (`ecosystem/fret-ui-ai/src/elements/workflow/node.rs`) is a shadcn-aligned chrome
  component, but it is not wired into a world-space layout/measure system.

Missing substrate:

- A reusable way to lay out *element tree children* at canvas-space positions under a `PanZoom2D`
  view (including hit-testing and invalidation under pan/zoom).

Recommendation:

- Treat this as a separate “declarative canvas world layer” workstream (likely `fret-ui-kit` or a
  dedicated ecosystem crate), because it is renderer + layout + hit-test sensitive.
- In the meantime, for interactive editors, prefer `fret-node`’s retained `NodeGraphCanvas`.

### Gap B — ReactFlow-like input filter knobs (`noWheel` / `noPan` / `.nokey`)

XYFlow includes fine-grained input filtering:

- Ignore wheel or pan when the event target is inside a `noWheelClassName` / `noPanClassName` subtree.
- Allow marquee selection “above” nodes by using pointer events capture, unless target is inside
  `.nokey`.

In Fret:

- Overlay elements can naturally “eat” pointer events by being above the canvas, but there is no
  standard recipe for “canvas is interactive unless event occurs inside an exempt subtree”.

Recommendation:

- Add an explicit recipe in `fret-canvas/ui` that composes:
  - pan/zoom base policy,
  - selection-on-drag gesture,
  - and a small “filter” interface (hit-test based, not class-name based).

### Gap C — Dashed strokes for paths (edge temporary)

AI Elements’ `edge.tsx` temporary edge uses `strokeDasharray: "5, 5"`.

In Fret today:

- Declarative `CanvasPainter::path` renders a tessellated stroke but does not expose dash patterns.

Recommendation:

- Decide whether dash belongs in:
  - the path service (tessellation) layer, or
  - a higher-level polyline approximation helper.

### Gap D — Edge markers (arrowheads) and richer routing primitives (component layer)

XYFlow supports marker end caps and multiple edge types (bezier, step, straight) and makes it easy
to supply a `markerEnd`.

In Fret today:

- `WorkflowEdge*` and `WorkflowConnection` are intentionally minimal canvas renderers.
- `fret-node` has much more advanced routing math, but it is not packaged as a general “edge kit”.

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

