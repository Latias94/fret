# Workspace TabStrip (Fearless Refactor v1) — Reference Notes

This file records reference implementations and what we intend to borrow (semantics, not code).

## Zed

- Workspace pane tab strip drag/drop:
  - `repo-ref/zed/crates/workspace/src/pane.rs`
  - Key ideas:
    - explicit end-drop target (`tab_bar_drop_target`)
    - `drag_over` border-based insert preview
    - close and activation policy separated from geometry

## dockview

- Drop target overlay / single-active droptarget discipline:
  - `repo-ref/dockview/packages/dockview-core/src/dnd/droptarget.ts`
  - Key ideas:
    - prevent "multiple overlays at once"
    - stable overlay model vocabulary

## gpui-component

- Tabs component + dock tab panel "last empty space" drop target:
  - `repo-ref/gpui-component/crates/ui/src/tab/tab_bar.rs`
  - `repo-ref/gpui-component/crates/ui/src/dock/tab_panel.rs`

## VS Code (external)

- Editor tabs preview/pinned semantics (preview editors):
  - Treat VS Code as behavioral inspiration for preview/dirty/pinned policy (not code).
  - Prefer referencing Zed for the in-repo, GPUI-adjacent semantics baseline; use VS Code only to
    sanity-check edge cases (e.g. when preview should be preserved vs replaced).

## Notes for Fret

- Reuse:
  - surface classification (`ecosystem/fret-ui-headless/src/tab_strip_surface.rs`)
  - click intent arbitration (`ecosystem/fret-ui-kit/src/headless/tab_strip_controller.rs`)
