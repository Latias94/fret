# Closeout Audit - 2026-04-21

Status: closed closeout record

This audit records the final closeout read for the ImUi key owner surface v1 lane.

Goal:

- verify whether the frozen M1 roster plus the M2 verdict still leave an active immediate
  key-owner design problem,
- separate the shipped helper-local shortcut answer from broader follow-on topics,
- and decide whether the lane should remain active or become a closed closeout record.

## Audited evidence

Core lane docs:

- `docs/workstreams/imui-key-owner-surface-v1/DESIGN.md`
- `docs/workstreams/imui-key-owner-surface-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
- `docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`
- `docs/workstreams/imui-key-owner-surface-v1/TODO.md`
- `docs/workstreams/imui-key-owner-surface-v1/MILESTONES.md`
- `docs/workstreams/imui-key-owner-surface-v1/EVIDENCE_AND_GATES.md`

Umbrella and reference docs:

- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`

Implementation / gate anchors:

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/lib.rs`

Validation run used for closeout:

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p0_key_owner_surface_follow_on immediate_mode_key_owner_surface_m2_no_new_surface_verdict_is_explicit --no-fail-fast`
- `cargo nextest run -p fret-imui menu_item_command_uses_command_metadata_shortcut_and_gating button_activate_shortcut_is_scoped_to_focused_button selectable_activate_shortcut_is_scoped_to_focused_item menu_item_activate_shortcut_is_scoped_to_focused_popup_item_and_preserves_arrow_nav begin_menu_activate_shortcut_is_scoped_to_focused_trigger begin_submenu_activate_shortcut_is_scoped_to_focused_trigger tab_item_activate_shortcut_is_scoped_to_focused_trigger checkbox_model_activate_shortcut_is_scoped_to_focused_checkbox switch_model_activate_shortcut_is_scoped_to_focused_switch combo_activate_shortcut_is_scoped_to_focused_trigger combo_model_activate_shortcut_is_scoped_to_focused_trigger --no-fail-fast`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 -m json.tool docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json > /dev/null`
- `python3 -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json > /dev/null`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Findings

### 1. The current helper-local shortcut seams already close the first-party key-owner demand for this cycle

The lane now has an explicit shipped answer:

- helper-local `activate_shortcut`,
- `shortcut_repeat`,
- `button_command[_with_options]`,
- and `menu_item_command[_with_options]`

already cover the first-party shortcut/key-owner pressure that the repo can currently prove.
The bounded 11-test targeted `fret-imui` floor keeps focus-scoped local activation executable
across the current direct pressable, popup-item, menu/submenu, tab, checkbox/switch, and combo
families.

Conclusion:

- the lane no longer has an active first-party key-owner gap that justifies additive helper growth
  today.

### 2. There is still no stronger first-party consumer pressure for a broader key-owner surface

The current repo scan across `apps/` and `ecosystem/` still finds the shortcut/key-owner seams
concentrated in the owner surface itself, its option/control implementations, and the lane-local
source-policy docs/tests.
`apps/fret-examples/src/imui_response_signals_demo.rs` remains the current proof/contract surface,
but it still does not force a generic item-local shortcut registration seam.

Conclusion:

- the lane should not invent a more generic surface only because the parity audit can name future
  depth.

### 3. The remaining pressure belongs to different owners and should not reopen this folder

What still remains after closeout is real, but it is not this lane's unfinished work:

1. `ResponseExt` lifecycle vocabulary
   - already belongs to the closed lifecycle follow-on.
2. Collection/pane proof breadth
   - already belongs to the closed collection/pane proof closeout record.
3. Broader menu/tab policy
   - already belongs to the separate trigger-response / policy chain.
4. Runtime keymap / IME arbitration
   - still belongs to ADR-backed runtime ownership, not this helper lane.
5. Runner/backend multi-window parity
   - remains active in `docs/workstreams/docking-multiwindow-imgui-parity/`.

Conclusion:

- this folder should stay closed and act as the closeout record for the current no-new-surface
  verdict rather than a standing backlog for future immediate convenience growth.

## Decision from this audit

Treat `imui-key-owner-surface-v1` as:

- closed for the current immediate key-owner / item-local shortcut ownership question,
- a closeout record for the shipped M2 no-new-surface verdict,
- and not the place to continue additive helper growth by default.

## Immediate execution consequence

From this point forward:

1. keep `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract
   surface unless a future narrower lane proves it insufficient,
2. keep the bounded 11-test targeted `fret-imui` package as the executable shortcut floor,
3. do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade here by
   default,
4. do not add a broader item-local shortcut registration seam here by default,
5. and start a different narrow lane with stronger first-party proof if future pressure still
   targets immediate key ownership.
