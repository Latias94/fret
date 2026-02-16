# Declarative Canvas World Layer v1 — TODO


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- XYFlow: https://github.com/xyflow/xyflow

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Active (workstream tracker)

This TODO list is scoped to the “nodes as element subtrees in a pan/zoom world” substrate.

See also:

- Workstream narrative: `docs/workstreams/canvas-world-layer-v1.md`
- XYFlow gap analysis: `docs/workstreams/xyflow-gap-analysis.md`

## Tracking format

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

## M0 — Spike (prove composition is viable)

- [x] CWL-M0-001 Add a minimal `CanvasWorld` host surface (API sketch; crate location TBD).
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_surface_panel`)
- [x] CWL-M0-002 Add a UI Gallery spike demo page:
  - place 2–3 node subtrees at world coords,
  - allow wheel zoom + pan drag,
  - show overlay chrome above the world layer.
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`
- [x] CWL-M0-003 Add a `fretboard diag` script capturing the spike page bundle.
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json`

## M1 — Correctness (hit-testing + scaling modes)

- [x] CWL-M1-001 Implement and document scale mode selection:
  - Scale-with-zoom (XYFlow-like),
  - Semantic-zoom (editor-like).
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`CanvasWorldScaleMode`)
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`
- [x] CWL-M1-002 Add a hit-testing gate:
  - click node under zoom still hits the node,
  - overlay buttons still receive input.
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json`

## M2 — Ergonomics (apps can build on it)

- [x] CWL-M2-001 Define a minimal “bounds reporting” seam so apps can implement:
  - fit-view to nodes,
  - selection-in-rect queries.
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`CanvasWorldBoundsStore`, `canvas_world_bounds_item`)
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (union display)
- [x] CWL-M2-002 Decide where selection-on-drag lives:
  - Decision: provide a world-layer-aware wrapper (marquee chrome must render above node subtrees).
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_surface_panel_with_marquee_selection`)
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (`ui-ai-cwl-marquee-anchor`)
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (`drag_pointer`)
- [x] CWL-M2-003 Add a small helper to compute `fit_view_to_canvas_rect(...)` from a set of bound keys.
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_fit_view_to_keys`)
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (Fit view button)
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (click `ui-ai-cwl-fit-view`)

## M3 — Interaction glue (optional, XYFlow-style)

This milestone is *not* a node-graph engine. It is a set of composable recipes/helpers so apps can
build XYFlow-like interactions on top of the world-layer substrate.

- [x] CWL-M3-001 Document an input arbitration recipe:
  - marquee selection should only start from “background” hits (not when clicking a node subtree).
  - Mechanism: `CanvasMarqueeSelectionProps::start_filter` (background-only gate).
  - Evidence: `ecosystem/fret-canvas/src/ui/pan_zoom.rs` (`CanvasMarqueeSelectionProps::start_filter`)
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (bounds-store-based filter)
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (marquee behavior not asserted yet)
  - Reference: `docs/workstreams/xyflow-gap-analysis.md` (Gap B)
- [x] CWL-M3-002 Provide a minimal node-drag recipe (app-owned model edits):
  - capture pointer in a node subtree,
  - translate screen delta → canvas delta,
  - update node canvas positions.
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (`ui-ai-cwl-node-a-drag-handle`)
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (assert `ui-ai-cwl-node-dragged`)
  - Reference: `repo-ref/xyflow/packages/system/src/xydrag/*`
- [x] CWL-M3-003 Provide a minimal connect-drag recipe surface:
  - start drag from a handle,
  - preview path,
  - validate + commit/cancel.
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (`ui-ai-cwl-node-a-source-handle`, `ui-ai-cwl-node-b-target-handle`, `ui-ai-cwl-connection-preview`)
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (drag `ui-ai-cwl-node-a-source-handle` → `ui-ai-cwl-node-b-target-handle`, assert `ui-ai-cwl-connection-committed`)
  - Note: `ui-ai-cwl-commit-connection` remains as a manual debug escape hatch, not a gate dependency.
  - Reference: `repo-ref/xyflow/packages/system/src/xyhandle/*`
- [ ] CWL-M3-004 Optional snap helpers (grid snap + snapline scaffolding).
  - Reference: `repo-ref/xyflow/packages/system/src/xydrag/*` (`snapGrid`, `snapToGrid`)
