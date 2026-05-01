# ImUi Menu/Tab Trigger Response Surface v1 - Evidence & Gates

Goal: keep the landed helper-owned trigger response surface tied to one current-behavior floor,
focused outward-surface proof, one demo/source gate, and one explicit lane boundary instead of
letting it drift back into the generic `imui` backlog.

## Evidence anchors (current)

- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/M0_BASELINE_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/TODO.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/MILESTONES.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/WORKSTREAM.json`
- `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/TODO.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `tools/gate_imui_workstream_source.py`
- `tools/gate_imui_facade_teaching_source.py`

## First-open repro surfaces

Use these to re-check the landed surface before any follow-on discussion:

1. Current helper behavior floor
   - `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger`
2. Landed outward-surface proof
   - `cargo nextest run -p fret-imui menu_and_submenu_response_report_toggle_and_trigger_edges tab_bar_response_reports_selected_change_and_trigger_edges`
3. Source-policy split + demo freeze
   - `python tools/gate_imui_workstream_source.py`
   - `python tools/gate_imui_facade_teaching_source.py`

## Current focused gates

### P0 source-policy gate

- `python tools/gate_imui_workstream_source.py`

This gate currently proves:

- the lane keeps the additive helper-owned trigger response contract explicit,
- the compatibility wrapper posture stays frozen,
- and the lifecycle lane still keeps this surface outside its narrower `ResponseExt` lane.

### Current helper behavior floor

- `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger`

This gate currently proves:

- top-level menu helpers still open/close and dispatch correctly,
- submenu helpers still expose the current nested open/expanded behavior,
- tab helpers still switch selected panels through the current model-driven path,
- and tab activation shortcuts still stay scoped to the focused trigger.

### Landed outward-surface proof

- `cargo nextest run -p fret-imui menu_and_submenu_response_report_toggle_and_trigger_edges tab_bar_response_reports_selected_change_and_trigger_edges`

This gate currently proves:

- menu/submenu additive response helpers report `open` / `toggled` / trigger edges,
- tab bar additive response reports `selected_changed` and per-trigger response access,
- and the landed surface stays facade-only instead of rewriting collection policy.

### Demo/source proof

- `python tools/gate_imui_facade_teaching_source.py`

This gate currently proves:

- the teaching demo shows `begin_menu_response_with_options`,
- the teaching demo shows `begin_submenu_response_with_options`,
- and the teaching demo shows `tab_bar_response_with_options` + per-trigger access.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 -m json.tool docs/workstreams/imui-menu-tab-trigger-response-surface-v1/WORKSTREAM.json > /dev/null`

## Residual routing after closure

Do not respond to future pressure by widening `fret-authoring::Response`, `crates/fret-ui`, or by
bundling richer menu/tab policy into this lane.
Start a narrower follow-on if future work is about broader menu/tab policy instead of this landed
helper-level response surface.
