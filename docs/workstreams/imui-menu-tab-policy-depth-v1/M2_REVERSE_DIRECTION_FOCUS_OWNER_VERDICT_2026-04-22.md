# M2 Reverse-Direction Focus Owner Verdict - 2026-04-22

Status: landed owner verdict within the active lane

## Purpose

Decide whether reverse-direction focus arbitration after top-level menubar `ArrowLeft` /
`ArrowRight` switching belongs in generic `fret-ui-kit::imui`, or whether it can close as a
shell/product-owned gap.

This note is narrower than active-menubar mnemonic posture:

- outer-scope mnemonic / roving posture is shell-owned by default,
- generic IMUI already owns trigger-local keyboard-open and in-menu top-level switching,
- but reverse switching from a later top-level menu back to an earlier sibling still lacked an
  explicit owner verdict.

## Evidence reviewed

Lane docs:

- `docs/workstreams/imui-menu-tab-policy-depth-v1/TODO.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/MILESTONES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_MENUBAR_KEYBOARD_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_MENUBAR_POPUP_OWNER_SYNC_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_MENUBAR_KEYBOARD_POSTURE_SLICE_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_ACTIVE_MENUBAR_MNEMONIC_ROVING_OWNER_VERDICT_2026-04-22.md`

Implementation and proof anchors:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

Focused local repro command:

- `cargo nextest run -p fret-imui begin_menu_horizontal_arrows_switch_active_top_level_menu --no-fail-fast`

## Findings

### 1. The reverse-direction gap reproduces without a shell/product menubar

The focused `fret-imui` harness already contains a two-trigger generic IMUI menubar test:

- focus the `File` trigger,
- open `File` with `ArrowDown`,
- switch forward to `Edit` with `ArrowRight`,
- then switch backward to `File` with `ArrowLeft`.

The shipped test freezes both-direction visibility and forward-direction content focus, but it
does not freeze reverse-direction content focus.

Local tightening of that harness showed the current failure shape:

- after `ArrowLeft`, the target `File` popup content can be visible for the switching frame,
- focus does not move to `File -> Open` during that frame,
- and on the following frame the menu can collapse with focus restored to a top-level trigger.

That reproduction uses only `fret-ui-kit::imui` and `fret-imui` test infrastructure, so the
failure is not caused by `fret::in_window_menubar`, workspace tabstrip policy, or shell mnemonic
state.

### 2. This is overlay/focus arbitration, not another mnemonic or roving question

The active-menubar owner verdict already keeps these behaviors outside generic IMUI:

- `Alt` / `F10` activation,
- mnemonic display and mnemonic-only opening,
- closed-state active-menubar `Escape` exit,
- and trigger-row roving/typeahead while no menu content is deployed.

Reverse-direction focus loss happens after a menu item inside deployed popup content handles a
plain horizontal arrow key. It therefore sits on the already-admitted generic keyboard slice:

- `menu_controls.rs` installs the content-item `ArrowLeft` / `ArrowRight` switch hook,
- `trigger_row.rs` updates the top-level active trigger and open model,
- `menu_family_controls.rs` projects menubar row state into popup visibility,
- and `popup_overlay.rs` / `window_overlays/render.rs` arbitrate open/close autofocus.

That path is generic IMUI popup/focus plumbing.

### 3. The current floor should not claim symmetric content-entry focus yet

The landed keyboard posture slice remains valid for:

- trigger-local `ArrowDown` / `ArrowUp` open,
- first-item focus on keyboard-open,
- both-direction top-level visibility switching,
- and forward-direction content focus when switching to a later sibling.

But it should not claim symmetric content-entry focus across render order yet. The reverse
direction has a different close/open ordering pressure: the newly targeted earlier sibling is
processed before the later sibling's hidden overlay cleanup has fully settled.

## Verdict

Reverse-direction focus arbitration belongs in generic IMUI.

Default owner:

- `ecosystem/fret-ui-kit::imui`
- the generic popup/overlay focus path shared by `begin_menu_with_options(...)`
- focused `ecosystem/fret-imui` tests

Not the default owner:

- `ecosystem/fret::in_window_menubar`
- `fret-workspace` tabstrip policy
- shell-level active-menubar mnemonic / roving surfaces
- runtime keymap or backend contracts

Do not close this as an accepted shell/product gap. It is a generic IMUI implementation gap that
needs a narrow follow-on fix, or a later explicit decision to keep the current floor asymmetric.

## Immediate consequence for this lane

The owner question is now closed, but the implementation gap remains open.

Next implementation slice should be narrow:

- preserve the existing forward-direction guarantees,
- make reverse-direction switching keep the target popup deployed,
- move focus to the target menu's entry item under keyboard modality,
- and avoid reopening outer-scope mnemonic / roving posture.

