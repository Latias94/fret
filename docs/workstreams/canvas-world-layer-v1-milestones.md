---
title: Declarative Canvas World Layer v1 — Milestones
status: draft
date: 2026-02-12
scope: ecosystem/fret-canvas/ui, ecosystem/fret-ui-kit, UI Gallery + diag gates
---

# Declarative Canvas World Layer v1 — Milestones

This is a compact milestone board for the “nodes as element subtrees under pan/zoom” substrate.

Source of truth TODOs: `docs/workstreams/canvas-world-layer-v1-todo.md`.

## Definition of done (v1)

We consider v1 usable when:

1. A UI Gallery demo proves composition and hit-testing correctness under pan/zoom.
2. There is at least one `fretboard diag` regression gate.
3. The layer boundary is clean (no policy leaks into `crates/fret-ui`).

## Milestones

### M0 — Spike demo (composition proof)

- A minimal host surface exists (API shape TBD).
- A UI Gallery page demonstrates nodes-as-elements under pan/zoom.
- One diag script captures the bundle for drift review.

### M1 — Correctness closure (hit-testing + scaling modes)

- Scale-with-zoom mode works (XYFlow-like).
- Semantic zoom mode works (editor-like).
- Click targets remain correct under zoom/pan.

### M2 — App ergonomics (bounds + selection seams)

- Apps can query/report node bounds for fit-view.
- Marquee selection integrates cleanly (recipe reuse or world-layer wrapper).

