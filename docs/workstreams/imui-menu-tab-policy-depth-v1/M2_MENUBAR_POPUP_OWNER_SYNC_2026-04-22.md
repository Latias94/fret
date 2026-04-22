# M2 Menubar Popup Owner Sync - 2026-04-22

Status: landed slice within the active lane

## Why this note exists

`begin_menu_with_options(...)` had already moved part of its behavior toward the popup overlay
contract so keyboard-open could focus the first menu item. That change exposed a deeper owner split:

- top-level menubar `trigger_row` policy still needed an out-of-band owner for hover-switch and
  active-trigger arbitration,
- while the popup overlay store still owned visibility, focus handoff, and dismissal.

Using the popup store `open` model as the direct `trigger_row` owner caused two regressions:

- hover-switch could open a sibling out-of-band, then lose it to popup-store stale pruning before
  `begin_menu_with_options(...)` could keep it alive,
- and submenu focus could dismiss the parent top-level popup because `FocusOutside` crossed overlay
  roots even though the submenu was still part of the same menu family.

## Landed decision

Top-level menubar IMUI now uses a two-step owner split:

1. `row_open` is the trigger-row owner for top-level menubar switching.
2. `popup_open` remains the overlay/popup owner.
3. `begin_menu_with_options(...)` projects `row_open -> popup_open` each frame.

Supporting rules landed with that split:

- `trigger_row` now render-syncs hover-switch timers from raw hover signals so top-level trigger
  switching still works even when popup barriers suppress ordinary pressable hover changes.
- top-level menu outward `DisclosureResponse` stays tied to popup visibility rather than the
  internal `row_open` arbitration model.
- top-level popup dismissal now preserves parent visibility across submenu focus transfers by
  ignoring `FocusOutside` while a submenu in the same popup family is open, but still performs the
  default close for other dismiss reasons.

## Evidence anchors

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

## Gates run for this slice

- `cargo nextest run -p fret-imui interaction_menu_tabs popup_hover --no-fail-fast`
- `git diff --check`

## Consequence for the lane

The earlier M2 keyboard audit blocker is now narrower than "top-level menu uses a different
focus/overlay contract". That owner split is resolved for the current IMUI floor. Remaining lane
questions stay where they were:

- whether richer submenu intent tuning belongs in generic IMUI,
- and whether roving / mnemonic posture should remain outside the shared helper family.
