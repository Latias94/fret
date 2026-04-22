# ImUi Menu/Tab Policy Depth v1 - TODO

Status: active execution lane
Last updated: 2026-04-22

## Lane setup

- [x] Create the lane as a narrow follow-on instead of reopening the umbrella or the closed
      trigger-response lanes.
- [x] Record an assumptions-first baseline audit.
- [x] Wire the lane into the repo-level workstream indexes.

## M0 - Baseline and owner freeze

- [x] Confirm that the closed trigger-response lanes only settled outward response shape and naming.
- [x] Confirm that the current generic IMUI family still stops at click-open menus and simple tab
      selection/panel switching.
- [x] Freeze the initial owner posture:
      generic menu/submenu policy may belong here, while richer tab overflow/reorder/close still
      needs an explicit owner audit.

## M1 - First slice freeze

- [x] Freeze one smallest landable slice instead of trying to solve the whole matrix at once:
      generic IMUI now owns top-level menubar hover-switch plus submenu hover-open / sibling
      hover-switch with a basic grace corridor as the current admitted policy-depth floor.
- [x] Freeze one first-party proof surface and one focused gate package for that slice.
- [ ] Keep the remaining owner questions explicit instead of widening the slice by accident:
      - richer submenu intent tuning beyond the current grace corridor,
      - roving or mnemonic posture.
      Current blocker: generic IMUI now admits trigger-local keyboard-open plus in-menu top-level
      left/right switching, but the remaining owner pressure is narrower:
      outer-scope active-menubar mnemonic / roving posture and reverse-direction focus arbitration
      still need an explicit verdict. See
      `M2_MENUBAR_KEYBOARD_POSTURE_SLICE_2026-04-22.md`.

## M2 - Land or close

- [x] Land the first justified slice with focused tests and source/evidence updates.
- [x] Close the top-level menubar popup-owner split so keyboard-open focus, top-level hover-switch,
      and submenu persistence can share one generic IMUI floor without popup-store churn.
      Result: `M2_MENUBAR_POPUP_OWNER_SYNC_2026-04-22.md` now records the landed `row_open ->
      popup_open` projection and the parent-menu dismiss fix for submenu focus transfers.
- [ ] Decide whether any richer submenu grace / intent tuning beyond the current corridor belongs
      in generic IMUI or closes on a shell/product owner verdict.
- [ ] Decide whether roving or mnemonic posture belongs in generic IMUI or should remain outside
      the shared helper family. The current evidence now says generic IMUI already owns a smaller
      keyboard floor (`ArrowDown` / `ArrowUp` open plus in-menu left/right switching), so the
      unresolved part is the outer-scope active-menubar shell posture and the remaining
      reverse-direction focus arbitration.
- [x] Run the explicit owner audit for tab overflow / scroll / reorder / close instead of growing
      generic IMUI by parity instinct alone.
      Result: `M2_TAB_OWNER_VERDICT_2026-04-22.md` now keeps editor-grade tabstrip policy in
      `fret-workspace::WorkspaceTabStrip` and out of generic `imui::tab_bar` by default.
- [ ] If the evidence shows the remaining pressure belongs to shell/product owners instead, close
      this lane on a no-new-generic-surface verdict rather than widening IMUI anyway.
