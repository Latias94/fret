# M2 Reverse-Direction Focus Handoff Slice - 2026-04-22

Status: landed slice within the active lane

## Why this note exists

`M2_REVERSE_DIRECTION_FOCUS_OWNER_VERDICT_2026-04-22.md` already closed the owner question:
reverse-direction focus arbitration after top-level menubar switching belongs in generic IMUI.

The remaining work was narrower than another owner split:

- `ArrowLeft` from a later top-level menu back to an earlier sibling reopened the target popup,
- the target popup could even move focus to its first item in the same frame,
- but hidden-popover cleanup from the closing later sibling still restored focus to the old
  trigger and erased the new entry focus.

That failure lived inside generic popup / focus teardown, not shell posture.

## Landed slice

Generic IMUI now preserves same-frame focus handoff when reverse-direction menubar switching
reopens an earlier sibling:

- keyboard-driven top-level `ArrowLeft` switching still reopens the earlier target popup,
- popup entry focus can replay after the popover root commit if the first attempt happened before
  the target item became live,
- and hidden popover teardown no longer restores focus back to the closing trigger when focus has
  already moved into another still-open overlay in the same window.

Supporting cleanup included:

- focus-outside dismissal only evaluates for popovers that were already open in the previous frame,
- and unseen popover/modal entries now clear stale `open` / `pending_initial_focus` bookkeeping
  when they fall out of the current request set.

## Evidence anchors

- `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/tests/cached_requests.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

## Gates run

- `cargo nextest run -p fret-ui-kit cached_requests dismissible_popover --no-fail-fast`
- `cargo nextest run -p fret-imui interaction_menu_tabs popup_hover --no-fail-fast`
- `git diff --check`

## Consequence for the lane

The remaining lane pressure is no longer menubar reverse-direction keyboard focus.

What remains open is narrower:

- whether richer submenu-intent tuning beyond the current grace corridor belongs in generic IMUI,
- or whether that pressure closes on a shell/product owner verdict instead of widening IMUI.
