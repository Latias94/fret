# ImUi Menu/Tab Trigger Response Surface v1 - Final Status

Status: closed closeout note
Last updated: 2026-04-13

Related:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

## Decision

This lane closes with a narrow additive outward response verdict:

- `begin_menu_response[_with_options]` now returns `DisclosureResponse`.
- `begin_submenu_response[_with_options]` now returns `DisclosureResponse`.
- `tab_bar_response[_with_options]` now returns `TabBarResponse`.

Compatibility wrappers remained in place at the moment this lane landed:

- `begin_menu[_with_options]` and `begin_submenu[_with_options]` still return `bool open`.
- `tab_bar[_with_options]` still behaves as the fire-and-forget helper surface for existing call
  sites.

A later cleanup now owns the naming-only follow-on:

- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/` now owns the
  compatibility-layer removal after this additive outward-response verdict landed.

The landed surface stays inside `ecosystem/fret-ui-kit::imui`, reuses `ResponseExt`, and does not
widen `fret-authoring::Response` or `crates/fret-ui`.

## Proof left behind

- Focused helper behavior floor:
  - `begin_menu_helper_toggles_popup_and_closes_after_command_activate`
  - `begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics`
  - `tab_bar_helper_switches_selected_panel_and_updates_selection_model`
  - `tab_item_activate_shortcut_is_scoped_to_focused_trigger`
- Focused outward-surface proof carried forward under the canonical helper test names:
  - `menu_and_submenu_helpers_report_toggle_and_trigger_edges`
  - `tab_bar_helper_reports_selected_change_and_trigger_edges`
- Demo/source proof:
  - `apps/fret-examples/src/imui_response_signals_demo.rs`
  - `imui_response_signals_demo_keeps_canonical_menu_tab_trigger_response_proof`

## Residual gap routing

This lane does not claim richer menu-bar/submenu/tab policy parity is complete.
If future pressure is about collection policy, key ownership, reorder/close semantics, or shell
product behavior, start a narrower follow-on instead of reopening this lane.
