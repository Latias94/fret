# Docking TabBar Fearless Refactor v1 (Reference Notes)

This file is a “where to look” index for upstream/reference implementations already vendored under
`repo-ref/`.

## Zed (editor semantics)

Key files:

- `repo-ref/zed/crates/workspace/src/pane.rs` (pinned tabs, preview semantics, tab bar rendering)
- `repo-ref/zed/crates/ui/src/components/tab_bar.rs` (TabBar layout: start/end children + scroll)

Patterns worth copying (at the policy layer):

- Pinned tabs as a first-class region (`pinned_tab_count`) and optional separate rows.
- Scroll-to-active behaviors (`ScrollHandle`).
- Strong “focus-neutral” behaviors when closing/moving (selection/focus invariants).

## gpui-component (wiring shape + explicit end space)

Key files:

- `repo-ref/gpui-component/crates/ui/src/dock/tab_panel.rs` (dock tab bar; has “empty space to drop at end”)
- `repo-ref/gpui-component/crates/ui/src/tab/tab.rs` (tab visuals + sizing hooks)

Patterns worth copying (as scaffolding, not semantics):

- `last_empty_space(...)` is an explicit end-drop surface (even if transparent).
- Split-zone detection is simple thresholds; for Fret we prefer “nearest edge + hysteresis”.

## dockview (overflow pipeline + header drop surfaces)

Key files:

- `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabsContainer.ts`
- `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabOverflowControl.ts`
- `repo-ref/dockview/packages/dockview-core/src/__tests__/dockview/components/titlebar/tabsContainer.spec.ts`

Patterns worth copying:

- Overflow as a pipeline (visible tabs + overflow dropdown list).
- “Header space” is treated as a drop surface, not just “between tabs”.

## What Fret is currently missing (most load-bearing gaps)

- Stable canonical index mapping for drop resolution under overflow (visible subset vs full list).
- Scripted (diag) gates for auto-scroll while dragging (docking currently has unit coverage; workspace does not).
- Pinned/preview semantics (must live in workspace/editor policy layer).
- Keyboard navigation + focus restore gates (should be scripted/gated, not hand-waved).
