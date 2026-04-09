---
title: Declarative Canvas World Layer v1 — Milestones
status: active
date: 2026-02-13
scope: ecosystem/fret-canvas/ui, ecosystem/fret-ui-kit, UI Gallery + diag gates
---

# Declarative Canvas World Layer v1 — Milestones

This is a compact milestone board for the “nodes as element subtrees under pan/zoom” substrate.

Source of truth TODOs: `docs/workstreams/canvas-world-layer-v1/canvas-world-layer-v1-todo.md`.

## Definition of done (v1)

We consider v1 usable when:

1. A UI Gallery demo proves composition and hit-testing correctness under pan/zoom.
2. There is at least one `fretboard-dev diag` regression gate.
3. The layer boundary is clean (no policy leaks into `crates/fret-ui`).

## Milestones

### M0 — Spike demo (composition proof)

Status: Done

- A minimal host surface exists.
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_surface_panel`)
- A UI Gallery page demonstrates nodes-as-elements under pan/zoom.
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs`
- One diag script captures the bundle for drift review.
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json`

### M1 — Correctness closure (hit-testing + scaling modes)

Status: Done

- Scale-with-zoom mode works (XYFlow-like).
- Semantic zoom mode works (editor-like).
- Click targets remain correct under zoom/pan.

### M2 — App ergonomics (bounds + selection seams)

Status: Done (with known gaps)

- Apps can query/report node bounds for fit-view.
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`CanvasWorldBoundsStore`, `canvas_world_bounds_item`)
- Fit-view glue exists for a set of active keys.
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_fit_view_to_keys`)
- Marquee selection integrates cleanly for world-layer nodes (chrome above nodes).
  - Evidence: `ecosystem/fret-canvas/src/ui/world_layer.rs` (`canvas_world_surface_panel_with_marquee_selection`)
  - Evidence: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (`drag_pointer`)

### M3 — Interaction glue (optional, XYFlow-style)

Status: In progress (partial)

- Background-only marquee start filtering exists (node subtrees do not trigger selection-on-drag).
- Node dragging recipe exists (app-owned position edits; canvas-space deltas).
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (`ui-ai-cwl-node-a-drag-handle`)
- Connection wiring exists (handles + preview); commit is currently gated via a deterministic helper button.
  - Evidence: `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` (`ui-ai-cwl-node-a-source-handle`, `ui-ai-cwl-node-b-target-handle`, `ui-ai-cwl-commit-connection`)
  - Gate: `tools/diag-scripts/ui-gallery-ai-canvas-world-layer-spike.json` (assert `ui-ai-cwl-connection-committed`)
- Input arbitration recipe exists (background-hit only marquee start).
