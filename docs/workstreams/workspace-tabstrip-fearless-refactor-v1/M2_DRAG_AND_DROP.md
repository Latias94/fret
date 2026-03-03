# Workspace TabStrip (Fearless Refactor v1) — Milestone 2 (Drag & Drop)

## Outcome

Editor-grade tab drag interactions with deterministic insert index semantics and stable preview
surfaces.

## Scope

- Reorder within a pane.
- Move tab across panes (within the same window).
- Drop at end surface (explicit header space) must remain stable.

Non-goals (v1):

- OS-level multi-window tab tearing from workspace tabs (may be dock-only in v1).
- Advanced split previews beyond the existing pane tree affordances (track separately).

## Exit criteria

- Diag gate(s):
  - reorder within a pane (insert index assertions)
  - move across panes (insert index assertions)
  - drag autoscroll (horizontal scroll while dragging)
  - drag-to-split preview (pane edge drop preview)
- Nextest tests cover:
  - canonical insert index mapping under overflow
  - "close does not activate" invariants during drag/drop sequences
