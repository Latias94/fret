# M1 Proof Roster Freeze — 2026-04-21

Purpose: freeze the current proof roster for `imui-key-owner-surface-v1` before the lane invents a
new dedicated shortcut/key-owner demo or quietly expands into unrelated immediate backlog.

## Evidence reviewed

- `docs/workstreams/imui-key-owner-surface-v1/DESIGN.md`
- `docs/workstreams/imui-key-owner-surface-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`

## Findings

### 1) Keep `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract surface

The repo does not yet have a stronger first-party key-owner demo than
`apps/fret-examples/src/imui_response_signals_demo.rs`.

This file is still the right first-open proof/contract surface because it already owns the
immediate response/contract posture, remains explicitly proof-first rather than showcase-first, and
stays closer to the shortcut/key-owner question than `imui_editor_proof_demo` or shell-mounted
surfaces.

At the same time, M1 should not overclaim what it proves:

- it is not yet a dedicated key-owner product proof,
- it does not yet demonstrate a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale surface,
- and it should therefore be read as the current proof/contract anchor, not as proof that no
  stronger first-party demo will ever be needed.

Conclusion:

- keep `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract
  surface.
- do not promote a new dedicated key-owner proof demo yet.
- do not promote `imui_editor_proof_demo` or `workspace_shell_demo` as shortcut/key-owner proof
  surfaces for this lane.

### 2) Freeze the current executable shortcut floor as one bounded targeted package

The current executable floor is already strong enough to avoid a vague full-suite dependency.
Keep this targeted package as the M1 floor:

- `menu_item_command_uses_command_metadata_shortcut_and_gating`
- `button_activate_shortcut_is_scoped_to_focused_button`
- `selectable_activate_shortcut_is_scoped_to_focused_item`
- `menu_item_activate_shortcut_is_scoped_to_focused_popup_item_and_preserves_arrow_nav`
- `begin_menu_activate_shortcut_is_scoped_to_focused_trigger`
- `begin_submenu_activate_shortcut_is_scoped_to_focused_trigger`
- `tab_item_activate_shortcut_is_scoped_to_focused_trigger`
- `checkbox_model_activate_shortcut_is_scoped_to_focused_checkbox`
- `switch_model_activate_shortcut_is_scoped_to_focused_switch`
- `combo_activate_shortcut_is_scoped_to_focused_trigger`
- `combo_model_activate_shortcut_is_scoped_to_focused_trigger`

This package is reviewable because it covers:

- command metadata shortcut display/gating,
- helper-local `activate_shortcut` on direct pressables,
- popup-item shortcut ownership under arrow-nav pressure,
- menu/submenu/tab trigger-local ownership,
- and model-backed combo/checkbox/switch ownership.

Conclusion:

- keep this bounded targeted `fret-imui` package as the current executable shortcut floor.
- do not depend on the full `cargo nextest run -p fret-imui` suite just to reason about this lane.

### 3) Freeze the explicit defer list for M1

The lane should keep these adjacent pressures out of scope at M1:

- `ResponseExt` lifecycle vocabulary
- collection/pane proof breadth
- broader menu/tab policy
- runtime keymap / IME arbitration
- runner/backend multi-window parity

These topics already have separate owners, closeout records, or ADR-backed runtime contracts.

Conclusion:

- M1 should freeze the current key-owner roster without widening into those neighboring problems.

## Execution consequence

From this note forward:

1. keep `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract
   surface,
2. keep the 11-test targeted `fret-imui` shortcut package as the current executable floor,
3. do not promote a new dedicated key-owner proof demo until M2 proves it is necessary,
4. and do not widen this lane into lifecycle, collection/pane, menu/tab policy, runtime
   arbitration, or runner/backend parity work.
