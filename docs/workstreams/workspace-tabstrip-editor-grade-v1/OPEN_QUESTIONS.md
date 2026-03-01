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

Status:
- Implemented (best-effort) in workspace pane drop-zone arbitration so split zones do not latch
  while the pointer is still over the tab strip row.

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

## Q4: What is the contract between “roving focus” and “tab cycling”?

Why it matters:
- Users expect arrow keys in the tab strip to move focus *visually* (in-order).
- Editors often bind `workspace.tab.next/prev` to cycle MRU (or at least not be strictly in-order).

Recommendation (default for v1):
- **ArrowLeft/ArrowRight** in the focused tab strip: in-order roving focus + **automatic activation**
  (APG Tabs-style).
- `workspace.tab.next/prev` commands: keep delegating to `WorkspaceTabs` `cycle_mode` (default MRU),
  and document this as a workspace policy surface (not a `fret-ui` runtime behavior).

Gate:
- Unit: `ecosystem/fret-workspace/tests/tab_strip_keyboard_roving_arrow_activates_tab.rs`
  (focus moves + `workspace.tab.activate.<id>` is dispatched).
