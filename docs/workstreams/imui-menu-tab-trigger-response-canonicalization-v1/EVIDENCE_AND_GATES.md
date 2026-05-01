# ImUi Menu/Tab Trigger Response Canonicalization v1 - Evidence & Gates

Goal: keep the helper trigger API cleanup tied to one reproducible call-site refactor surface, one
focused behavior gate, one demo/source gate, and one explicit lane boundary.

## Evidence anchors (current)

- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/TODO.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/MILESTONES.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/WORKSTREAM.json`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/lib.rs`

## First-open repro surfaces

1. Current call-site spread
   - `rg -n "begin_menu_with_options\\(|begin_menu\\(|begin_submenu_with_options\\(|begin_submenu\\(|tab_bar_with_options\\(|\\.tab_bar\\(" -g '*.rs'`
2. Current helper behavior floor
   - `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger`
3. Current outward-surface proof
   - `cargo nextest run -p fret-imui menu_and_submenu_helpers_report_toggle_and_trigger_edges tab_bar_helper_reports_selected_change_and_trigger_edges`
4. Current demo/source proof
   - `python tools/gate_imui_facade_teaching_source.py`

## Intended gates for this lane

### Focused IMUI behavior floor

- `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger`

### Focused canonical response proof

- `cargo nextest run -p fret-imui menu_and_submenu_helpers_report_toggle_and_trigger_edges tab_bar_helper_reports_selected_change_and_trigger_edges`

### Demo/source proof

- `python tools/gate_imui_facade_teaching_source.py`

### Lane hygiene gates

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_surface_follow_on immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_canonicalization_follow_on`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 -m json.tool docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/WORKSTREAM.json > /dev/null`
