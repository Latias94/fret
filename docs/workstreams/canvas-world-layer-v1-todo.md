# Declarative Canvas World Layer v1 — TODO

Status: Draft (workstream tracker)

This TODO list is scoped to the “nodes as element subtrees in a pan/zoom world” substrate.

See also:

- Workstream narrative: `docs/workstreams/canvas-world-layer-v1.md`
- XYFlow gap analysis: `docs/workstreams/xyflow-gap-analysis.md`

## Tracking format

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

## M0 — Spike (prove composition is viable)

- [ ] CWL-M0-001 Add a minimal `CanvasWorld` host surface (API sketch; crate location TBD).
- [ ] CWL-M0-002 Add a UI Gallery spike demo page:
  - place 2–3 node subtrees at world coords,
  - allow wheel zoom + pan drag,
  - show overlay chrome above the world layer.
- [ ] CWL-M0-003 Add a `fretboard diag` script capturing the spike page bundle.

## M1 — Correctness (hit-testing + scaling modes)

- [ ] CWL-M1-001 Implement and document scale mode selection:
  - Scale-with-zoom (XYFlow-like),
  - Semantic-zoom (editor-like).
- [ ] CWL-M1-002 Add a hit-testing gate:
  - click node under zoom still hits the node,
  - overlay buttons still receive input.

## M2 — Ergonomics (apps can build on it)

- [ ] CWL-M2-001 Define a minimal “bounds reporting” seam so apps can implement:
  - fit-view to nodes,
  - selection-in-rect queries.
- [ ] CWL-M2-002 Decide where selection-on-drag lives:
  - reuse `fret-canvas/ui` marquee recipe,
  - or provide a world-layer aware wrapper.

