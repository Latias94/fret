# ImUi Menu/Tab Policy Depth v1 - TODO

Status: closed lane
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
      hover-switch with an end-to-end enforced grace corridor as the current admitted
      policy-depth floor.
- [x] Freeze one first-party proof surface and one focused gate package for that slice.
- [x] Keep the remaining owner questions explicit instead of widening the slice by accident:
      - richer submenu intent tuning beyond the current grace corridor,
      - or a no-new-generic-surface verdict for that extra submenu tuning pressure.
      Current blocker: `M2_ACTIVE_MENUBAR_MNEMONIC_ROVING_OWNER_VERDICT_2026-04-22.md` now keeps
      outer-scope active-menubar mnemonic / roving posture shell-owned by default,
      `M2_REVERSE_DIRECTION_FOCUS_OWNER_VERDICT_2026-04-22.md` kept reverse-direction focus inside
      generic IMUI, and `M2_REVERSE_DIRECTION_FOCUS_HANDOFF_SLICE_2026-04-22.md` has now landed
      that keyboard handoff fix, so the only explicit remaining policy-depth question was richer
      submenu intent beyond the current grace corridor.
      Result: `CLOSEOUT_AUDIT_2026-04-22.md` now closes that remaining question on a
      no-new-generic-surface verdict.

## M2 - Land or close

- [x] Land the first justified slice with focused tests and source/evidence updates.
- [x] Close the top-level menubar popup-owner split so keyboard-open focus, top-level hover-switch,
      and submenu persistence can share one generic IMUI floor without popup-store churn.
      Result: `M2_MENUBAR_POPUP_OWNER_SYNC_2026-04-22.md` now records the landed `row_open ->
      popup_open` projection and the parent-menu dismiss fix for submenu focus transfers.
- [x] Decide whether any richer submenu grace / intent tuning beyond the current corridor belongs
      in generic IMUI or closes on a shell/product owner verdict.
      Result: `CLOSEOUT_AUDIT_2026-04-22.md` now closes this lane because the current generic IMUI
      floor already covers the first-party submenu outcomes this repo can justify, while the
      remaining pressure is only micro-tuning without a new generic consumer proof.
- [x] Prove that the current grace corridor is actually enforced end-to-end in generic IMUI rather
      than only existing in the submenu primitive.
      Result: `M2_SUBMENU_GRACE_CORRIDOR_PROOF_SLICE_2026-04-22.md` now records the landed fix for
      the IMUI hover-hook override plus helper-local hover bypass, and the focused
      `begin_submenu_helper_defers_sibling_switch_inside_grace_corridor` /
      `begin_submenu_helper_safe_corridor_cancels_close_timer` gates now lock both sibling-switch
      deferral and close-timer cancellation on the current floor.
- [x] Decide whether outer-scope active-menubar roving / mnemonic posture belongs in generic IMUI
      or should remain outside the shared helper family.
      Result: `M2_ACTIVE_MENUBAR_MNEMONIC_ROVING_OWNER_VERDICT_2026-04-22.md` now keeps Alt/F10
      activation, mnemonic display/open, closed-state Escape exit, and trigger-row roving/typeahead
      in shell-owned `fret::in_window_menubar`-style surfaces by default.
- [x] Decide whether reverse-direction focus arbitration after top-level left/right switching
      belongs in generic IMUI or can close as an accepted current floor gap.
      Result: `M2_REVERSE_DIRECTION_FOCUS_OWNER_VERDICT_2026-04-22.md` now keeps the owner inside
      generic IMUI because the focused `fret-imui` repro fails without any shell/product
      menubar surface in play.
- [x] Land a narrow generic IMUI fix for reverse-direction top-level switching so keyboard focus
      enters the reopened earlier sibling instead of dropping to trigger-level fallback or
      collapsing on the next frame.
      Result: `M2_REVERSE_DIRECTION_FOCUS_HANDOFF_SLICE_2026-04-22.md` now records the landed
      same-frame handoff fix in popup teardown / initial-focus replay.
- [x] Run the explicit owner audit for tab overflow / scroll / reorder / close instead of growing
      generic IMUI by parity instinct alone.
      Result: `M2_TAB_OWNER_VERDICT_2026-04-22.md` now keeps editor-grade tabstrip policy in
      `fret-workspace::WorkspaceTabStrip` and out of generic `imui::tab_bar` by default.
- [x] If the evidence shows the remaining pressure belongs to shell/product owners instead, close
      this lane on a no-new-generic-surface verdict rather than widening IMUI anyway.
      Result: `CLOSEOUT_AUDIT_2026-04-22.md` now treats this lane as historical closeout evidence
      and requires any future submenu-intent growth to reopen as a narrower follow-on.
