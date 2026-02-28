# Workspace TabStrip (editor-grade) v1 — Open questions

This file captures decisions that affect long-term behavior and scriptability. Prefer answering
these with contracts + gates rather than ad-hoc implementation tweaks.

## Q1: Should “drag to split” be allowed while the pointer is still in the tab strip row?

Why it matters:
- Users expect “tab strip row” drags to mean reorder / move-to-pane.
- Allowing split zones in the same row tends to cause accidental splits near the pane edges.

Recommendation:
- **No**. Treat the tab strip row as a “center-only” zone for split preview purposes.
- Require the pointer to leave the tab strip row (into the pane content bounds) before split zones
  can activate.

Gate:
- Keep `workspace-shell-demo-tab-drag-to-split-right-drop-preview-screenshot` (screenshot) and add
  an invariants-based split gate once preview routing is stable.

## Q2: Should diagnostics scripts rely on pixel deltas for editor-grade drag interactions?

Why it matters:
- `set_window_inner_size` is best-effort; actual window bounds can differ across runners/OS.
- Large deltas can overshoot window bounds, preventing edge-margin logic from triggering.

Recommendation:
- Prefer `drag_to` when a stable target exists.
- Prefer `drag_pointer_until` for “find the edge preview” style gates.
- Only use raw `pointer_move` deltas for short, local motions.

## Q3: Where should “keep drag session position fresh” live long-term?

Today:
- The tab strip interaction layer updates `DragSession::position` defensively during drag moves.

Recommendation:
- Treat this as a **mechanism** concern. Long-term, move “drag session position tracking” into the
  DnD/runtime layer (so non-tab surfaces don’t depend on tab-strip-local pointer capture).
- Until then, keep the current workaround and gate it via scripts.

