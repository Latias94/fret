---
title: Declarative Canvas World Layer v1 (XYFlow-style nodes as element subtrees)
status: draft
date: 2026-02-12
scope: ecosystem/fret-canvas/ui, ecosystem/fret-ui-kit, apps (UI Gallery demos + diag gates)
---

# Declarative Canvas World Layer v1

This workstream explores a reusable “world layer” substrate for **canvas-space element subtrees**
similar to XYFlow/ReactFlow’s mental model:

- nodes are normal UI subtrees (DOM in web land; element trees in Fret),
- edges live in a world-space paint layer (SVG in web land; `CanvasPainter` in Fret),
- pan/zoom transforms both together.

This is intentionally **not** a goal to recreate ReactFlow’s API surface. The goal is to unlock the
same composition outcome in Fret while keeping layering clean.

Related docs:

- XYFlow gap analysis (why this matters): `docs/workstreams/xyflow-gap-analysis.md`
- AI Elements workflow chrome (current state): `docs/workstreams/ai-elements-port.md`

## Problem statement

Today, Fret can already ship editor-grade workflow editing via retained engines (e.g. `fret-node`).
However, the “nodes are element subtrees in a transformed world” composition style is still missing
as a reusable substrate.

For AI Elements parity, this shows up as:

- `WorkflowCanvas` is a host surface (pan/zoom + overlay slot),
- `WorkflowNode` is a chrome component,
- but there is no canonical mechanism to place a set of `WorkflowNode` element trees at
  canvas-space positions under a `PanZoom2D` view.

## Non-goals (v1)

- No full node-graph engine (editing policy remains app-owned).
- No renderer-level new primitives (no path marker system, no dash tessellation changes).
- No virtualization guarantees (v1 is correctness + composition; perf work follows).

## Key outcomes we want

1. **Composition**: allow nodes as element subtrees (shadcn/AI chrome) placed by canvas-space
   coordinates.
2. **Correct hit-testing** under pan/zoom (pointer targets match visuals).
3. **Deterministic input arbitration** between world layer and overlays (toolbars, controls, etc.).
4. **Two scaling modes** (configurable):
   - **Scale-with-zoom** (XYFlow-like; nodes zoom visually with the world),
   - **Semantic zoom** (editor-like; nodes stay constant in screen px while their positions follow
     the zoomed canvas mapping).

## Design sketch (proposed seams)

At a high level:

- A `CanvasWorld` element owns:
  - a `Model<PanZoom2D>` (view state),
  - a background + edge paint pass (`CanvasPainter`),
  - a node subtree slot (children) positioned in world coordinates.
- Each node subtree provides:
  - a canvas-space position,
  - an identity key (`test_id` + stable keying),
  - optional bounds reporting for selection/fit-view seams.

Open questions (to be answered by a spike):

- Whether we should express the transform as `render_transform` vs “pre-transform positions” in
  layout (tradeoffs for hit-testing and measurement).
- How to expose a stable “world-space layout contract” without leaking renderer details into
  `crates/fret-ui`.

## Current v0 status (2026-02-12)

Implemented:

- `fret-canvas/ui` composition helper: `ecosystem/fret-canvas/src/ui/world_layer.rs`
  (`canvas_world_surface_panel`)
- Scale modes:
  - `CanvasWorldScaleMode::ScaleWithZoom` (world subtree render-transformed)
  - `CanvasWorldScaleMode::SemanticZoom` (world subtree screen-space; callers position nodes via
    `CanvasWorldPaintCx::canvas_to_screen`)
- UI Gallery spike: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`
- Diag gate: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json`

Known limitations (v0):

- Bounds for the world transform are derived from `layout_query_bounds(...)` (last-frame), so
  sudden resizes can have one-frame transform mismatch. v1 should remove this by computing the
  transform from current-frame bounds (requires a deeper runtime seam).

## Quality gates

Minimum:

- UI Gallery spike page demonstrating nodes-as-elements under pan/zoom.
- At least one `fretboard diag` gate verifying:
  - pan/zoom moves the node subtrees predictably,
  - overlay exemption regions still prevent wheel-pan where expected.

## Milestones + TODO

- Milestones: `docs/workstreams/canvas-world-layer-v1-milestones.md`
- TODO tracker: `docs/workstreams/canvas-world-layer-v1-todo.md`
