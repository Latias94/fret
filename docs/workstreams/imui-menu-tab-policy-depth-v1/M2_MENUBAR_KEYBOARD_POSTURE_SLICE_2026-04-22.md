# M2 Menubar Keyboard Posture Slice - 2026-04-22

Status: landed slice within the active lane

## Why this note exists

`M2_MENUBAR_KEYBOARD_AUDIT_2026-04-22.md` correctly blocked the naive plan of plugging
`trigger_row` keyboard helpers straight into generic `begin_menu_with_options(...)` while top-level
menus still had a different owner/focus contract from popup/context menus.

`M2_MENUBAR_POPUP_OWNER_SYNC_2026-04-22.md` resolved enough of that split to admit a smaller,
generic IMUI keyboard slice without reopening runtime contracts or shell-owned active-menubar
choreography.

## Landed slice

Generic IMUI menubars now admit this keyboard posture:

- focused top-level `begin_menu_with_options(...)` triggers open on `ArrowDown` / `ArrowUp`,
- keyboard-open still routes through popup-menu autofocus, so first-item entry focus remains part
  of the same generic menu root,
- popup menu roots rendered under a menubar now inherit `ImUiMenubarPolicyState`,
- and focused menu items inside an open top-level menu can switch to sibling top-level menus on
  `ArrowLeft` / `ArrowRight`.

Implementation detail that matters for correctness:

- popup overlays now suppress one close-auto-focus cycle while a top-level horizontal switch is in
  flight, so the closing menu does not immediately steal focus back from the newly opened sibling.

## Evidence anchors

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

## Gates run

- `cargo nextest run -p fret-imui interaction_menu_tabs popup_hover --no-fail-fast`
- `git diff --check`

## Residual gap after this slice

This does **not** close the whole menubar keyboard question.

Still unresolved:

- whether outer-scope active-menubar posture (`Alt`, `F10`, mnemonic-only opening, closed-state
  active-menubar exit) belongs in generic IMUI or remains shell-owned,
- and whether reverse-direction switching from a later top-level trigger back to an earlier one
  should guarantee immediate content-entry focus, rather than the current render-order-sensitive
  close/open result.

The current focused test package freezes:

- `ArrowDown` / `ArrowUp` keyboard-open + first-item focus,
- top-level left/right switching visibility in both directions,
- and forward-direction content focus when switching to a later sibling.

## Consequence for the lane

The remaining question is no longer "can generic IMUI host any menubar keyboard posture at all?"

It is now narrower:

- generic IMUI already owns trigger-local keyboard-open and in-menu top-level switching,
- while fuller active-menubar shell posture and the remaining reverse-switch focus arbitration
  still need an explicit owner verdict.
