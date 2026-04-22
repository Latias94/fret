# ImUi Menu/Tab Policy Depth v1 - Evidence & Gates

Goal: keep the broader menu/submenu/tab policy discussion tied to the current shipped floor and a
single narrow follow-on, instead of reopening already-closed response-surface work.

## Evidence anchors

- `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_TAB_OWNER_VERDICT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_MENUBAR_KEYBOARD_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/reference-stack-ui-behavior.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- `ecosystem/fret-workspace/src/tab_strip/interaction.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/sub_trigger.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
- `ecosystem/fret-workspace/tests/tab_strip_overflow_menu_lists_overflowed_tabs.rs`
- `ecosystem/fret-workspace/tests/tab_strip_focus_restore_after_close_command.rs`
- `ecosystem/fret-workspace/tests/tab_strip_pinned_boundary_has_test_id.rs`
- `ecosystem/fret-workspace/tests/tab_strip_keyboard_roving_arrow_activates_tab.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_demo.cpp`

## First-open repro surfaces

1. Immediate interaction showcase
   - `cargo run -p fret-demo --bin imui_interaction_showcase_demo`
2. Immediate response-signal proof
   - `cargo run -p fret-demo --bin imui_response_signals_demo`
3. Shell/product comparison proof
   - `cargo run -p fret-demo --bin workspace_shell_demo`

Read these in order:

- showcase/response demos prove the current generic IMUI floor,
- workspace shell and workspace-tabstrip evidence prove where richer tabstrip product behavior
  already has a different owner.

## Current gate package

- `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_menu_helper_hover_switches_top_level_popup_after_trigger_hover_delay begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics begin_submenu_helper_hover_opens_submenu_after_pointer_entry begin_submenu_helper_hover_switches_sibling_after_open_delay menu_and_submenu_helpers_report_toggle_and_trigger_edges tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_bar_helper_reports_selected_change_and_trigger_edges --no-fail-fast`
- `cargo nextest run -p fret-workspace tab_strip_overflow_menu_lists_overflowed_tabs tab_strip_focus_restore_after_close_command tab_strip_pinned_boundary_has_test_id tab_strip_keyboard_roving_arrow_activates_tab --no-fail-fast`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-menu-tab-policy-depth-v1/WORKSTREAM.json > /dev/null`

This gate package currently proves:

- the shipped menu family now covers click-open triggers, top-level menubar hover-switch, submenu
  hover-open, sibling submenu hover-switch with a basic grace corridor, nested submenus, and
  outward trigger responses,
- the shipped tab family still covers simple selection and panel switching,
- the richer editor-grade tabstrip outcomes already stay executable in `fret-workspace`,
- and the lane remains correctly indexed and anchored as a narrow follow-on.

## Remaining gap after the current landed floor

Still missing before this lane can close:

- an explicit verdict on whether any richer submenu-intent tuning beyond the current grace
  corridor stays generic,
- or an explicit owner verdict that leaves that pressure to shell/product layers,
- plus any explicit decision on whether roving / mnemonic posture stays generic or not.
- the current keyboard blocker is now explicit:
  generic IMUI top-level `begin_menu_with_options(...)` still uses a different focus/overlay
  contract than popup/context menu surfaces, so richer menubar keyboard policy is not yet a
  straight plumbing task.
