# M2 No-New-Surface Verdict - 2026-04-21

Purpose: decide whether the current key-owner lane should widen the immediate surface beyond the
shipped helper-local shortcut seams, or explicitly stop at a no-new-surface verdict.

## Evidence reviewed

- `docs/workstreams/imui-key-owner-surface-v1/DESIGN.md`
- `docs/workstreams/imui-key-owner-surface-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Findings

### 1) The current helper-local seams already cover the first-party key-owner demand that Fret can prove today

Fret already ships the immediate seams that the current first-party proof can actually justify:

- helper-local `activate_shortcut`,
- `shortcut_repeat` as the explicit repeat-policy seam,
- `button_command[_with_options]`,
- and `menu_item_command[_with_options]`.

The bounded 11-test shortcut floor already proves focus-scoped local activation on direct
pressables, popup items, menu/submenu triggers, tabs, and combo families without widening
`crates/fret-ui` or global shortcut ownership.

Conclusion:

- keep the current helper-local `activate_shortcut` + `shortcut_repeat` + `button_command` /
  `menu_item_command` seams as the shipped immediate key-owner answer for this cycle.

### 2) There is still no stronger first-party consumer pressure for a broader key-owner surface

A current repo scan across `apps/` and `ecosystem/` outside `fret-imui` tests shows the shortcut /
key-owner vocabulary is still concentrated in:

- `ecosystem/fret-ui-kit/src/imui.rs`,
- option types and helper/control implementations,
- and lane source-policy docs/tests.

There is still no stronger first-party consumer pressure for a broader key-owner surface.
The repo does not currently have a first-party app/demo that clearly needs a generic item-local
shortcut registration seam or a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale facade.
`apps/fret-examples/src/imui_response_signals_demo.rs` remains the current proof/contract surface,
but it still does not force a broader API than the shipped helper-local seams.

Conclusion:

- do not treat parity-audit desire alone as proof that a new immediate facade is warranted.

### 3) The remaining pressure is still about proof depth or adjacent policy, not the current owner split

The Dear ImGui parity audit can still name missing depth, but the open pressure does not yet prove
that the current owner split is wrong:

- `ResponseExt` lifecycle vocabulary already belongs to the closed lifecycle lane,
- collection/pane proof breadth already belongs to the closed collection/pane proof lane,
- broader menu/tab policy already belongs to the separate trigger-response / policy chain,
- runtime keymap / IME arbitration still belongs to ADR-backed runtime ownership,
- and runner/backend parity still belongs to the docking multi-window lane.

Conclusion:

- keep lifecycle vocabulary, collection/pane proof breadth, broader menu/tab policy, runtime
  keymap / IME arbitration, and runner/backend parity in their separate lanes.

## Verdict

M2 closes on a no-new-surface verdict.

Do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade.
Do not add a broader item-local shortcut registration seam.
Keep the current helper-local `activate_shortcut` + `shortcut_repeat` + `button_command` /
`menu_item_command` seams as the shipped answer until stronger first-party proof says otherwise.

## Immediate execution consequence

From this point forward:

1. keep `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract
   surface unless a future narrower lane proves it insufficient,
2. keep the bounded 11-test targeted `fret-imui` package as the executable shortcut floor,
3. do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade,
4. do not add a broader item-local shortcut registration seam,
5. and reopen this question only if stronger first-party proof exceeds the current demo/test
   floor.
