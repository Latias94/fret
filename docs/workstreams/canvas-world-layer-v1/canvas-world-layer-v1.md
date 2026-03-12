---
title: Declarative Canvas World Layer v1 (XYFlow-style nodes as element subtrees)
status: active
date: 2026-02-13
scope: ecosystem/fret-canvas/ui, ecosystem/fret-ui-kit, apps (UI Gallery demos + diag gates)
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- XYFlow: https://github.com/xyflow/xyflow

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# Declarative Canvas World Layer v1

This workstream explores a reusable “world layer” substrate for **canvas-space element subtrees**
similar to XYFlow/ReactFlow’s mental model:

- nodes are normal UI subtrees (DOM in web land; element trees in Fret),
- edges live in a world-space paint layer (SVG in web land; `CanvasPainter` in Fret),
- pan/zoom transforms both together.

This is intentionally **not** a goal to recreate ReactFlow’s API surface. The goal is to unlock the
same composition outcome in Fret while keeping layering clean.

Related docs:

- XYFlow gap analysis (why this matters): `docs/workstreams/standalone/xyflow-gap-analysis.md`
- AI Elements workflow chrome (current state): `docs/workstreams/ai-elements-port/ai-elements-port.md`

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

## Current v0 status (2026-02-13)

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

## Next (M2) — Ergonomics (bounds + selection seams)

To make this substrate usable for real apps (AI Elements “workflow” experiences), we still need:

1. **Bounds reporting seam**
   - Status: Implemented (M2-001)
     - `ecosystem/fret-canvas/src/ui/world_layer.rs`:
       - `CanvasWorldBoundsStore`
       - `canvas_world_bounds_item(...)`
   - Status: Implemented (M2-003) fit-view helper from active keys:
     - `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_fit_view_to_keys`)
     - Demo: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (`ui-ai-cwl-fit-view`)
   - helpers exist to map bounds between screen space and canvas space for:
     - `fitView` math (`fit_view_to_canvas_rect`),
     - selection-in-rect queries,
     - minimap extents.

2. **Selection-on-drag integration**
   - Status: Implemented (M2-002) via a world-layer-aware wrapper:
     - `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_surface_panel_with_marquee_selection`)
   - Rationale: canvas-paint marquee chrome would render behind node element subtrees.
   - Diagnostics note: the UI Gallery spike is gated with a real pointer drag:
     - `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (`drag_pointer`)

3. **Input arbitration recipes**
   - document (and ideally provide helpers for) common patterns:
     - “node subtree has an inner scroll area: wheel scroll should not pan the canvas”,
     - “text input inside a node: drag/select gestures should not start a canvas pan”.

## Next (M3) — Interaction glue (optional)

The v1 substrate is intentionally “mechanism-first”: it enables nodes-as-elements under pan/zoom,
but it does not provide a node-graph editing engine.

If we want an XYFlow/ReactFlow-like *component-first workflow editor* experience on top of the
world-layer substrate, the remaining work is mostly **interaction glue**:

- Node dragging (screen-space pointer deltas → canvas-space position edits).
- Connection dragging (handles, strict/loose targeting, validity checks, previews).
- Selection model updates (click vs marquee, shift-add/remove, connected-edges selection policy).
- Optional snap tuning (grid snap, snaplines).

Current state:

- A minimal node-drag recipe exists as a UI Gallery spike:
  - Drag handle: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`
    (`ui-ai-cwl-node-a-drag-handle`)
  - Gate: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json`
    (assert `ui-ai-cwl-node-dragged`)
- Connection wiring exists as a UI Gallery spike (handles + preview); commit is gated via a real drag gesture:
  - Handles: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`
    (`ui-ai-cwl-node-a-source-handle`, `ui-ai-cwl-node-b-target-handle`)
  - Gate: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json`
    (drag `ui-ai-cwl-node-a-source-handle` → `ui-ai-cwl-node-b-target-handle`, assert
    `ui-ai-cwl-connection-committed`)

Notes:

- `ui-ai-cwl-commit-connection` still exists as a manual debug escape hatch but is no longer used by
  the gate script.
- Marquee selection integration exists (`canvas_world_surface_panel_with_marquee_selection`), but
  background-drag gating is not asserted by the current script yet (input arbitration between world
  element subtrees and the underlying pan/zoom surface remains a v1 ergonomics gap).

Input arbitration note:

- XYFlow-style marquee selection should only start from “background” hits (pane), not when the down
  event is within a node subtree.
- In Fret, a practical recipe is to provide a `CanvasMarqueeSelectionProps::start_filter` that
  checks the down position against an app-owned node bounds store (see
  `CanvasWorldBoundsStore` + the UI Gallery spike).

Reference implementations:

- XyFlow substrate: `repo-ref/xyflow/packages/system/src/*` (`xydrag`, `xyhandle`, `xyresizer`)
- fret-node (retained engine today): `ecosystem/fret-node` and `docs/workstreams/standalone/fret-node-xyflow-parity.md`

## Quality gates

Minimum:

- UI Gallery spike page demonstrating nodes-as-elements under pan/zoom.
- At least one `fretboard diag` gate verifying:
  - pan/zoom moves the node subtrees predictably,
  - overlay exemption regions still prevent wheel-pan where expected.

## Milestones + TODO

- Milestones: `docs/workstreams/canvas-world-layer-v1/canvas-world-layer-v1-milestones.md`
- TODO tracker: `docs/workstreams/canvas-world-layer-v1/canvas-world-layer-v1-todo.md`
