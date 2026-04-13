# ImUi Menu/Tab Trigger Response Surface v1 - Evidence & Gates

Goal: keep the helper-owned trigger response-surface decision tied to one current-behavior floor,
one source-policy gate, and one explicit lane boundary instead of letting it drift back into the
generic `imui` backlog.

## Evidence anchors (current)

- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/M0_BASELINE_AUDIT_2026-04-13.md`
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
- `apps/fret-examples/src/lib.rs`

## First-open repro surfaces

Use these before debating any API shape in the abstract:

1. Current helper behavior floor
   - `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger`
2. Source-policy split freeze
   - `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_surface_follow_on`

## Current focused gates

### P0 source-policy gate

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_surface_follow_on`

This gate currently proves:

- the new follow-on exists as a separate lane,
- the lifecycle lane keeps this topic deferred,
- and the umbrella still treats implementation-heavy P0 work as narrow follow-ons instead of one
  giant backlog.

### Current helper behavior floor

- `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger`

This gate currently proves:

- top-level menu helpers still open/close and dispatch correctly,
- submenu helpers still expose the current nested open/expanded behavior,
- tab helpers still switch selected panels through the current model-driven path,
- and tab activation shortcuts still stay scoped to the focused trigger.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 -m json.tool docs/workstreams/imui-menu-tab-trigger-response-surface-v1/WORKSTREAM.json > /dev/null`

## Missing gates before closure

Before claiming this lane is closed, add:

- one explicit proof of the final verdict:
  either no-new-API source proof or a real outward-surface test/demo proof,
- and any focused interaction gates required by the final outward surface if one lands.

Do not respond to that gap by widening `fret-authoring::Response`, `crates/fret-ui`, or by
bundling richer menu/tab policy into this lane.
