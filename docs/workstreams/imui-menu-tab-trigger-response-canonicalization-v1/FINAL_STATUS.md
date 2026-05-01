# ImUi Menu/Tab Trigger Response Canonicalization v1 - Final Status

Status: closed closeout note
Last updated: 2026-04-13

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Decision

This lane closes on the canonical naming cleanup:

- `begin_menu[_with_options]` now returns `DisclosureResponse`.
- `begin_submenu[_with_options]` now returns `DisclosureResponse`.
- `tab_bar[_with_options]` now returns `TabBarResponse`.
- The duplicate `begin_menu_response[_with_options]`,
  `begin_submenu_response[_with_options]`, and `tab_bar_response[_with_options]` aliases are
  removed.

The shipped helper surface stays inside `ecosystem/fret-ui-kit::imui` and does not widen
`fret-authoring::Response` or `crates/fret-ui`.

## Proof left behind

- Focused helper behavior floor:
  - `begin_menu_helper_toggles_popup_and_closes_after_command_activate`
  - `begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics`
  - `tab_bar_helper_switches_selected_panel_and_updates_selection_model`
  - `tab_item_activate_shortcut_is_scoped_to_focused_trigger`
- Focused canonical response proof:
  - `menu_and_submenu_helpers_report_toggle_and_trigger_edges`
  - `tab_bar_helper_reports_selected_change_and_trigger_edges`
- Demo/source proof:
  - `python tools/gate_imui_facade_teaching_source.py`
  - `immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_canonicalization_follow_on`

## Historical routing

- Read `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md` as the
  historical answer to "should helper-owned outward trigger responses exist at all?"
- Read this closeout as the later answer to "which helper names remain canonical after that
  additive surface landed?"
- If future work broadens into menu/tab policy, key ownership, closable/reorderable tabstrip
  product behavior, or shell-level concerns, start another narrow follow-on instead of reopening
  this naming cleanup lane.
