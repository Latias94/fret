# Workspace TabStrip (editor-grade) v1 — TODO

This TODO list is scoped to this workstream folder and is intended to keep the refactor landable.

## Diagnostics + gates

- [x] Stabilize cross-pane move gate (`workspace-shell-demo-tab-cross-pane-move-to-end`).
- [x] Stabilize reorder smoke gate (`workspace-shell-demo-tab-reorder-first-to-end-smoke`).
- [x] Add an end-drop reorder gate under overflow (`workspace-shell-demo-tab-reorder-first-to-end-overflow-smoke`).
- [x] Stabilize drag-to-split gates:
  - [x] `workspace-shell-demo-tab-drag-to-split-right`
  - [x] `workspace-shell-demo-tab-drag-to-split-right-drop-preview-screenshot`
- [x] Add a smoke gate for “split preview suppressed while pointer stays in the tab strip row”:
  - [x] `workspace-shell-demo-tab-drag-to-split-right-row-suppressed-smoke`
- [x] Add an overflow activation smoke gate (`workspace-shell-demo-tab-overflow-activate-hidden-smoke`).
- [x] Add an overflow menu close gate (does not activate, tolerates small pointer jitter): `workspace-shell-demo-tab-overflow-close-does-not-activate`
- [x] Add a close-button “click jitter does not start drag” gate (`workspace-shell-demo-tab-close-button-does-not-start-drag`).
- [x] Promote a minimal suite that runs in < 30s locally (`diag-hardening-smoke-workspace`) and keep it green.
- [ ] Add a non-screenshot invariants-based split gate (post-split layout assertions + tab ownership), once split preview routing is stable.

## Modularization (M1)

- [x] Split tab strip implementation into modules under `ecosystem/fret-workspace/src/tab_strip/`.
- [ ] Keep shrinking `ecosystem/fret-workspace/src/tab_strip/mod.rs` by moving more render-only code into
  `widgets.rs`/`layouts.rs` and by keeping the interaction surface small and auditable.
- [ ] Keep public surface stable (no upstream callers rewritten unnecessarily).
- [ ] Ensure `cargo nextest run -p fret-workspace` stays green throughout.

## Behavior parity (editor-grade)

- [x] Drag-to-split: define when split zones are allowed while dragging a tab (tab strip row vs content area).
  - Evidence: `ecosystem/fret-workspace/src/panes.rs` (`tab_strip_row_contains_pointer` split suppression)
  - Gate: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right-row-suppressed-smoke.json`
- [x] End-drop resolution uses canonical order under overflow (does not depend on tab bounds).
- [x] Overflow activation scrolls the newly active tab into view (gated).
- [ ] Overflow menu: deterministic scroll-into-view under repeated resize/scroll (stress).
- [ ] Close policies: close button vs middle click vs keyboard (policy-layer ownership documented).
- [x] Focus restore after close when the tab strip is focused (unit test gate).
- [x] Keyboard focus transfer into the tab strip (`workspace.pane.focus_tab_strip`) (unit test gate).
- [x] Default keybinding for `workspace.pane.toggle_tab_strip_focus` (`Ctrl+F6`).
- [x] Escape exits the focused tab strip (`workspace.pane.focus_content`) (unit test gate).
- [x] `focus_content` / `Ctrl+F6` can exit even if no return target was recorded (pane content fallback).
- [x] Keyboard nav: baseline roving ArrowLeft/ArrowRight auto-activates (unit test gate).
- [x] Keyboard nav: decide MRU vs in-order for `workspace.tab.next/prev` and lock with gates (default: MRU).
  - Evidence: `ecosystem/fret-workspace/src/tabs.rs` (`TabCycleMode::Mru` default + unit tests).
  - Evidence: `ecosystem/fret-workspace/src/commands.rs` (default keybindings for `workspace.tab.next/prev`).
- [x] Make tab strip hit-test surfaces explicit (tab row vs header space vs overflow control / scroll controls) and gate it via unit tests.
- [x] Adopt `WorkspacePaneContentFocusTarget` in workspace shells (real pane content), so exit fallback works in demos.
  - Evidence: `apps/fret-examples/src/workspace_shell_demo.rs`.
  - Evidence: `ecosystem/fret/src/workspace_shell.rs` (golden-path shell registers pane content target).
