# Workspace Shell TabStrip (Fearless Refactor v1) — TODO

This file is an execution checklist for the design in `DESIGN.md`.

## Setup / Inventory

- [ ] Inventory current tab strip usage:
  - [ ] Workspace shell: `apps/fret-ui-gallery/src/driver/chrome.rs`
  - [ ] Workspace shell composition: `ecosystem/fret/src/workspace_shell.rs`
  - [ ] Docking tab bars: `ecosystem/fret-docking/src/dock/*`
- [ ] List all existing tab-related commands and decide ownership:
  - activate, close, close others, close left/right
  - move active before/after target (reorder)
  - pin/unpin
  - preview open/commit
- [ ] Confirm `test_id` conventions for:
  - tab strip root
  - each tab trigger (and close button)
  - overflow control + overflow list entries

## M1 — Overflow dropdown/list

- [x] Decision: use “overflowed-only” list (dockview-style). See `OPEN_QUESTIONS.md`.
- [x] Add overflow computation based on measured tab rects + viewport.
- [x] Add overflow control button that appears only when overflowing.
- [x] Add overflow list panel:
  - [x] stable `test_id` for open button and entries
  - [x] select entry activates tab and scrolls into view
  - [ ] optional close button in overflow list
- [ ] Gates:
  - [x] test: overflow membership stable under resize + scroll offset changes
  - [ ] diag script: open overflow list, select an overflowed tab, assert active

## M2 — Pinned boundary

- [x] Decision: use `pinned_tab_count` boundary (Zed-like). See `OPEN_QUESTIONS.md`.
- [ ] Add drop targets to allow moving tabs into/out of the pinned region.
- [ ] Optional: add “separate pinned row” when pinned + unpinned exist.
- [ ] Gates:
  - [ ] test: pin/unpin preserves active and maintains order
  - [ ] diag: drag a tab across pinned boundary

## M3 — Preview tabs

- [ ] Define preview policy contract:
  - [ ] when preview opens, reuse existing preview tab slot
  - [ ] when committed, becomes normal tab
  - [ ] if preview disabled, always open normal tabs
- [ ] Decide how preview is represented in UI (icon/italic/indicator).
- [ ] Gates:
  - [ ] test: open sequence replaces preview tab
  - [ ] test: commit preview preserves active tab

## M4 — Kernel extraction + docking reuse

- [ ] Decide the extraction target:
  - [ ] keep in `ecosystem/fret-workspace` as `tab_strip_kernel` module, or
  - [ ] new crate `ecosystem/fret-editor-chrome` consumed by both workspace and docking.
- [ ] Define kernel interfaces:
  - [ ] inputs: rects, pointer position, viewport size, state snapshot
  - [ ] outputs: intents (activate/reorder/move/split/scroll)
- [ ] Refactor `WorkspaceTabStrip` to call kernel helpers (no behavior duplication).
- [ ] Apply the same kernel to docking tab bars.
- [ ] Gates:
  - [ ] unit tests for hit testing → insertion side matrix
  - [ ] integration test for cross-pane move intents

## M5 — Drag-to-split integration

- [ ] Define split target geometry (edge thresholds + hysteresis).
- [ ] Emit `SplitPane` intents from kernel; execute split in workspace/docking policy layer.
- [ ] Gates:
  - [ ] diag script: drag tab to edge and drop, assert split + tab moved

## Notes / Evidence anchors

- Zed tab bar/pane reference: `repo-ref/zed/crates/workspace/src/pane.rs`
- dockview overflow list reference:
  - `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabs.ts`
  - `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabsContainer.ts`
- gpui-component dock tab panel reference: `repo-ref/gpui-component/crates/ui/src/dock/tab_panel.rs`
