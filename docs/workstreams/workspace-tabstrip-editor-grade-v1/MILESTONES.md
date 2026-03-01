# Workspace TabStrip (editor-grade) — Milestones

This workstream is intentionally incremental. Each milestone must keep the system landable with
diagnostics gates (scripts) so refactors remain fearless and reversible.

## M1 — Modularize + baseline gates

Outcomes:
- `WorkspaceTabStrip` implementation is split into small modules (view/interaction/geometry/kernel).
- Stable `test_id` anchors exist for scriptability (root + tabs + pinned boundary + overflow).
- At least 2 promoted diagnostics scripts gate:
  - reorder within a single strip (invariants-first):
    - `workspace-shell-demo-tab-reorder-first-to-end-smoke` (currently: first -> after second)
  - drag-to-split “drop preview” in workspace shell demo
    - initial: `workspace-shell-demo-tab-drag-to-split-right-drop-preview-screenshot`
    - follow-up: replace with an invariants-based gate once the drop preview snapshot surface is stable
  - middle-click close behavior (smoke):
    - `workspace-shell-demo-tab-middle-click-close-smoke`
  - close button behavior (smoke):
    - `workspace-shell-demo-tab-close-button-closes-tab-smoke`

Acceptance:
- `python3 tools/check_diag_scripts_registry.py` passes.
- `cargo run -p fretboard -- diag run ... --launch -- <workspace shell demo>` passes reliably.

## M2 — Editor-grade behaviors (Zed-style expectations)

Outcomes:
- Pinned tabs behavior is complete:
  - configurable “separate pinned row” vs “single row + boundary”
  - reorder and pin/unpin are unambiguous and gated
- Overflow menu is complete and stable under resize/scroll.
- Keyboard navigation is solid:
  - roving focus
  - MRU vs in-order cycling integrates with `WorkspaceTabs`

Acceptance:
- Diagnostics scripts cover pinned/unpinned and overflow edge cases.
- No regressions in docking arbitration demos that reuse shared drag primitives.

## M3 — Polish + perf hygiene

Outcomes:
- No layout jitter when tabs change title/dirty/preview state (stable width rules).
- Auto-scroll near edges is smooth and deterministic during drag.
- Optional screenshot baselines exist for key visuals (only where invariants are insufficient).

Acceptance:
- A small perf/correctness gate exists for “worst-frame while dragging tabs” (optional, if needed).
- All new scripts are promoted (suite membership) and stable across DPI scaling.
