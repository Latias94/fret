# Workspace Shell TabStrip (Fearless Refactor v1) — Evidence & Gates

This document turns milestones into concrete, reviewable evidence and regression protection.

## Stable `test_id` conventions (proposal)

We should keep selectors stable across refactors. Suggested shapes:

- Root:
  - `{root}` (via `WorkspaceTabStrip::test_id_root`)
- Per-tab:
  - tab trigger: `{tab_prefix}-{tab_id}` (via `WorkspaceTabStrip::tab_test_id_prefix`)
  - tab chrome container: `{tab_prefix}-{tab_id}.chrome`
  - close button: `{tab_prefix}-{tab_id}.close`
  - dirty indicator: `{tab_prefix}-{tab_id}.dirty`
- Overflow:
  - overflow button: `{root}.overflow_button`
  - overflow entry: `{root}.overflow_entry.{tab_id}`
  - overflow entry close (future): `{root}.overflow_entry.{tab_id}.close`
- Drop targets:
  - end-of-strip: `{root}.drop_end`
  - pinned boundary: `{root}.drop_pinned_boundary`
  - pinned row border (if separate row, TODO): `{root}.drop_pinned_row`
- Drag-to-split (workspace panes):
  - drop preview overlay: `workspace-pane-{pane_id}.drop_preview.{zone}`
    - `{zone}`: `left | right | up | down | center`

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
- Edge auto-scroll:
  - pointer near left/right edges produces deterministic scroll deltas.
  - prefer shared helper coverage: `ecosystem/fret-dnd/src/scroll.rs` (`compute_autoscroll_x/y`)
- Preview tab:
  - open previewable item replaces existing preview tab slot.
- MRU:
  - toggling MRU between two most recent remains stable under close/reorder.

### UI wiring gates (runtime behavior)

Prefer nextest tests for “hard” behaviors that do not require real rendering:

- Focus stability:
  - pointer down on tab does not steal focus from an existing focus target.
- Focus transfer:
  - `workspace.pane.focus_tab_strip` focuses the active tab in the focused pane.
  - `workspace.pane.focus_tab_strip` works when focus starts outside the pane subtree (shell scope).
  - `workspace.pane.focus_content` restores the pre-tabstrip focus target after keyboard use of the strip.
  - Default keybinding suggestion: `Ctrl+F6` bound to `workspace.pane.toggle_tab_strip_focus`
    (apps can override via keymap layering).
  - Roving keyboard navigation:
    - arrow keys move roving focus and activate correct tab.

## Diag gates (interaction-heavy)

For drag/drop and overflow UX, scripted `fretboard-dev diag` gates are preferred:

### Script gates (current)

- Suite:
  - `diag-hardening-smoke-workspace` (promoted via `tools/diag-scripts/index.json`)
- Workspace shell demo:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-overflow-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-cross-pane-move-to-end.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right-drop-preview-screenshot.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-activate-hidden-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json`
    (gates `source_kind=pointer` for `workspace.tab.close.doc-a-0`)
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-mru-fallback-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-boundary-toggle-smoke.json`

### Determinism knobs

When adding scripts, prefer:

- fixed frame delta (`--fixed-frame-delta-ms 16`) for animation stability
- stable `test_id` targeting over pixel coordinates where possible

## Evidence anchors (what reviewers should look at)

For each milestone PR, include 1–3 anchors:

- key functions (kernel ops / adapter wiring)
- tests / diag script IDs
- demo surface (UI Gallery page and/or `fretboard-dev dev` command)

Current anchors:

- Workspace tab strip adapter: `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- Tab strip interaction kernel (WIP): `ecosystem/fret-workspace/src/tab_strip/kernel.rs`
- Focus transfer gate: `ecosystem/fret-workspace/tests/pane_focus_tab_strip_command_focuses_active_tab.rs`
- Shell scope gate: `ecosystem/fret-workspace/tests/workspace_command_scope_focus_tab_strip_from_outside_pane.rs`
- Exit tab strip gate: `ecosystem/fret-workspace/tests/workspace_command_scope_focus_content_restores_previous_focus.rs`
- Toggle focus gate: `ecosystem/fret-workspace/tests/workspace_command_scope_toggle_tab_strip_focus_toggles_between_content_and_tab_strip.rs`
- Toggle (multi-pane) gate: `ecosystem/fret-workspace/tests/workspace_command_scope_toggle_tab_strip_focus_multi_pane_returns_to_last_non_tabstrip_focus.rs`

Reference anchors:

- Zed pinned/preview/drop targets: `repo-ref/zed/crates/workspace/src/pane.rs`
- dockview overflow list pipeline:
  - `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabs.ts`
  - `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabsContainer.ts`
