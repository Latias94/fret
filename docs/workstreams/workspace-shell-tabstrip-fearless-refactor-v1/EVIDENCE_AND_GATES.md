# Workspace Shell TabStrip (Fearless Refactor v1) — Evidence & Gates

This document turns milestones into concrete, reviewable evidence and regression protection.

## Stable `test_id` conventions (proposal)

We should keep selectors stable across refactors. Suggested shapes:

- Root:
  - `workspace-tabstrip`
- Per-tab:
  - tab trigger: `workspace-tabstrip-tab-{tab_id}`
  - tab chrome container: `workspace-tabstrip-tab-{tab_id}.chrome`
  - close button: `workspace-tabstrip-tab-{tab_id}.close`
  - dirty indicator: `workspace-tabstrip-tab-{tab_id}.dirty`
- Overflow:
  - overflow button: `workspace-tabstrip-overflow-button`
  - overflow panel: `workspace-tabstrip-overflow-panel`
  - overflow entry: `workspace-tabstrip-overflow-entry-{tab_id}`
  - overflow entry close: `workspace-tabstrip-overflow-entry-{tab_id}.close`
- Drop targets:
  - end-of-strip: `workspace-tabstrip-drop-end`
  - pinned boundary: `workspace-tabstrip-drop-pinned-boundary`
  - pinned row border (if separate row): `workspace-tabstrip-drop-pinned-row`

Notes:

- Prefer `{tab_id}` over indices to keep automation stable under reorder.
- When `{tab_id}` contains slashes/spaces, normalize (e.g. replace non-alnum with `_`).

## Test gates (unit/integration)

### Core state invariants (pure logic)

Add tests close to the kernel/module (or in `ecosystem/fret-workspace/tests/` if kept there):

- Reorder intent correctness matrix:
  - given rects + pointer positions, compute `(target_id, insertion_side)` deterministically.
- “Drop end” target:
  - dropping in empty space produces “insert at end” intent.
- Pinned boundary:
  - pin/unpin updates `pinned_tab_count` and preserves active tab.
- Preview tab:
  - open previewable item replaces existing preview tab slot.
- MRU:
  - toggling MRU between two most recent remains stable under close/reorder.

### UI wiring gates (runtime behavior)

Prefer nextest tests for “hard” behaviors that do not require real rendering:

- Focus stability:
  - pointer down on tab does not steal focus from an existing focus target.
- Roving keyboard navigation:
  - arrow keys move roving focus and activate correct tab.

## Diag gates (interaction-heavy)

For drag/drop and overflow UX, scripted `fretboard diag` gates are preferred:

### Suggested script names (to be added when implementing milestones)

- `tools/diag-scripts/workspace-tabstrip-overflow-open-select.json`
- `tools/diag-scripts/workspace-tabstrip-drag-reorder.json`
- `tools/diag-scripts/workspace-tabstrip-cross-pane-move.json`
- `tools/diag-scripts/workspace-tabstrip-drag-to-split.json`
- `tools/diag-scripts/workspace-tabstrip-pinned-boundary.json`

### Determinism knobs

When adding scripts, prefer:

- fixed frame delta (`--fixed-frame-delta-ms 16`) for animation stability
- stable `test_id` targeting over pixel coordinates where possible

## Evidence anchors (what reviewers should look at)

For each milestone PR, include 1–3 anchors:

- key functions (kernel ops / adapter wiring)
- tests / diag script IDs
- demo surface (UI Gallery page and/or `fretboard dev` command)

Reference anchors:

- Zed pinned/preview/drop targets: `repo-ref/zed/crates/workspace/src/pane.rs`
- dockview overflow list pipeline:
  - `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabs.ts`
  - `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabsContainer.ts`

