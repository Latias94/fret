# ImUi Menu/Tab Policy Depth v1 - Milestones

Status: closed lane
Last updated: 2026-04-22

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why this is a new narrow follow-on,
- the current menu/tab shipped floor is documented,
- and the initial owner split is explicit before implementation starts.

Primary evidence:

- `M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- Closed on 2026-04-21 via `M0_BASELINE_AUDIT_2026-04-21.md`.

## M1 - First slice and proof roster freeze

Exit criteria:

- one smallest policy-depth slice is chosen,
- the lane states whether that slice belongs in generic IMUI or elsewhere,
- and one first-party proof surface plus one focused gate package are frozen.

Primary evidence:

- `DESIGN.md`
- `M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `M2_TAB_OWNER_VERDICT_2026-04-22.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

Current status:

- Closed on 2026-04-22 via `M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`.
- The landed first slice is narrower than the original broad menu-depth ask:
  generic IMUI now carries top-level menubar hover-switch plus submenu hover-open / sibling
  hover-switch with a basic grace corridor.
- `M2_TAB_OWNER_VERDICT_2026-04-22.md` now keeps editor-grade tab overflow / scroll / reorder /
  close in `fret-workspace`, so the remaining generic IMUI follow-on questions are narrower:
  richer submenu intent tuning and roving / mnemonic posture.

## M2 - Land or close with an explicit verdict

Exit criteria:

- the first justified slice lands with focused tests and evidence updates,
- or the lane closes with an explicit no-new-generic-surface verdict if the remaining pressure is
  better owned by shell/product layers.

Primary evidence:

- `M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `M2_TAB_OWNER_VERDICT_2026-04-22.md`
- `M2_MENUBAR_KEYBOARD_POSTURE_SLICE_2026-04-22.md`
- `M2_ACTIVE_MENUBAR_MNEMONIC_ROVING_OWNER_VERDICT_2026-04-22.md`
- `M2_REVERSE_DIRECTION_FOCUS_OWNER_VERDICT_2026-04-22.md`
- `M2_REVERSE_DIRECTION_FOCUS_HANDOFF_SLICE_2026-04-22.md`
- `M2_SUBMENU_GRACE_CORRIDOR_PROOF_SLICE_2026-04-22.md`
- `CLOSEOUT_AUDIT_2026-04-22.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- future landed status note or closeout note

Current status:

- Closed on 2026-04-22 via `CLOSEOUT_AUDIT_2026-04-22.md`.
- A first landed generic IMUI floor now exists:
  top-level menubar hover-switch plus submenu hover-open / sibling hover-switch with an
  end-to-end enforced grace corridor, including void-corridor close-timer cancellation, locked by
  focused `fret-imui` tests.
- `M2_MENUBAR_POPUP_OWNER_SYNC_2026-04-22.md` now closes the top-level owner split between
  `trigger_row` switching and popup overlay visibility:
  keyboard-open focus, top-level hover-switch, Escape close, and submenu parent persistence now
  share one generic IMUI floor instead of fighting popup-store stale pruning.
- `M2_MENUBAR_KEYBOARD_POSTURE_SLICE_2026-04-22.md` now lands a smaller generic IMUI keyboard
  slice on top of that owner split:
  focused triggers open on `ArrowDown` / `ArrowUp`, and focused menu items can switch top-level
  menus on `ArrowLeft` / `ArrowRight`.
- `M2_ACTIVE_MENUBAR_MNEMONIC_ROVING_OWNER_VERDICT_2026-04-22.md` now keeps outer-scope
  active-menubar mnemonic / roving posture in shell-owned `in_window_menubar`-style surfaces
  instead of growing generic IMUI by primitive availability alone.
- `M2_REVERSE_DIRECTION_FOCUS_OWNER_VERDICT_2026-04-22.md` now closes the owner split for the
  remaining reverse-direction keyboard problem:
  the failure reproduces in the focused generic `fret-imui` harness, so it remains a generic IMUI
  implementation gap rather than a shell/product posture question.
- `M2_REVERSE_DIRECTION_FOCUS_HANDOFF_SLICE_2026-04-22.md` now lands that remaining generic IMUI
  keyboard slice:
  reverse-direction switching can hand focus to the reopened earlier sibling without hidden-popover
  cleanup restoring the old trigger over the new popup entry focus.
- `M2_SUBMENU_GRACE_CORRIDOR_PROOF_SLICE_2026-04-22.md` now closes the gap between primitive
  submenu grace capability and IMUI end-to-end behavior:
  IMUI hover-query hooks no longer overwrite submenu primitive hover handlers, and helper-local
  hover state no longer bypasses primitive delay / grace ownership; the focused IMUI proof now
  locks both sibling-switch deferral inside the grace polygon and safe-corridor close-timer
  cancellation while moving through submenu-side void.
- `CLOSEOUT_AUDIT_2026-04-22.md` now closes the remaining question on a no-new-generic-surface
  verdict:
  the current generic floor is sufficient for the first-party evidence on hand, and any future
  wider submenu-intent pressure must reopen as a narrower follow-on.
