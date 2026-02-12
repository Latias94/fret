# Declarative Canvas World Layer v1 — TODO

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
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (simulated commit + wiring)
  - Note: `fretboard diag` currently cannot drag; the spike page includes a simulate button for gating.
- [x] CWL-M2-003 Add a small helper to compute `fit_view_to_canvas_rect(...)` from a set of bound keys.
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_fit_view_to_keys`)
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (Fit view button)
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (click `ui-ai-cwl-fit-view`)
