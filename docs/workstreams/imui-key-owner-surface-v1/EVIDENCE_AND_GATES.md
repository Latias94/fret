# ImUi Key Owner Surface v1 - Evidence & Gates

Goal: keep the key-owner lane tied to one current repro set, one explicit gate floor, and one
bounded evidence set, then close it explicitly once the repo proves that broader immediate
key-owner surface is not justified.

## Evidence anchors (current)

- `docs/workstreams/imui-key-owner-surface-v1/DESIGN.md`
- `docs/workstreams/imui-key-owner-surface-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
- `docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`
- `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-key-owner-surface-v1/TODO.md`
- `docs/workstreams/imui-key-owner-surface-v1/MILESTONES.md`
- `docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/lib.rs`

## First-open repro surfaces

Use these before reading older historical `imui` notes in depth:

1. Current proof/contract demo surface
   - `cargo run -p fret-demo --bin imui_response_signals_demo`
2. Current targeted shortcut/key-owner floor
   - `cargo nextest run -p fret-imui menu_item_command_uses_command_metadata_shortcut_and_gating button_activate_shortcut_is_scoped_to_focused_button selectable_activate_shortcut_is_scoped_to_focused_item menu_item_activate_shortcut_is_scoped_to_focused_popup_item_and_preserves_arrow_nav begin_menu_activate_shortcut_is_scoped_to_focused_trigger begin_submenu_activate_shortcut_is_scoped_to_focused_trigger tab_item_activate_shortcut_is_scoped_to_focused_trigger checkbox_model_activate_shortcut_is_scoped_to_focused_checkbox switch_model_activate_shortcut_is_scoped_to_focused_switch combo_activate_shortcut_is_scoped_to_focused_trigger combo_model_activate_shortcut_is_scoped_to_focused_trigger --no-fail-fast`

Current shipped closeout stance:

- proof/contract surface: `imui_response_signals_demo`
- executable shortcut floor: the bounded 11-test targeted `fret-imui` package above
- shipped verdict: no new `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate surface
  and no broader item-local shortcut registration seam

Reopen this closeout only if stronger first-party proof shows that these surfaces are insufficient.

## Current focused gates

### Lane-local source-policy gate

- `python tools/gate_imui_workstream_source.py`

This gate currently proves:

- the repo keeps the new key-owner lane explicit,
- the lane now closes on the M2 no-new-surface verdict,
- the lane stays separate from lifecycle, collection/pane, and menu/tab policy work,
- and the umbrella notes the closed key-owner follow-on as its own narrow owner/problem.

### Current shortcut/key-owner interaction floor

- `cargo nextest run -p fret-imui menu_item_command_uses_command_metadata_shortcut_and_gating button_activate_shortcut_is_scoped_to_focused_button selectable_activate_shortcut_is_scoped_to_focused_item menu_item_activate_shortcut_is_scoped_to_focused_popup_item_and_preserves_arrow_nav begin_menu_activate_shortcut_is_scoped_to_focused_trigger begin_submenu_activate_shortcut_is_scoped_to_focused_trigger tab_item_activate_shortcut_is_scoped_to_focused_trigger checkbox_model_activate_shortcut_is_scoped_to_focused_checkbox switch_model_activate_shortcut_is_scoped_to_focused_switch combo_activate_shortcut_is_scoped_to_focused_trigger combo_model_activate_shortcut_is_scoped_to_focused_trigger --no-fail-fast`

This floor currently proves:

- command metadata shortcut display/gating works on the current immediate menu surface,
- helper-local `activate_shortcut` remains focus-scoped on direct pressables, popup items, menu
  triggers, submenu triggers, tabs, and combo families,
- and the repo already has executable proof for local shortcut ownership before adding new facade
  surface.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json > /dev/null`

## Expected next gate additions

No further gate additions are required while this lane stays closed.
Reopen only if stronger first-party proof shows that the current helper-local seams are
insufficient.
