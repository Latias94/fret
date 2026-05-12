---
title: Fret Mechanism Harness v1 Evidence and Gates
status: active
date: 2026-05-12
---

# Evidence and Gates

## Synthetic Harness Gates

```powershell
cargo test --profile dev-fast -p fret-mechanism-harness --lib mechanism_metrics_can_assert_non_geometry_facts -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_layout_dirty_invalidation_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_scroll_handle_invalidation_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_environment_view_cache_invalidation_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_layout_primitives_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_hit_test_routing_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_pointer_occlusion_routing_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_focus_barrier_routing_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-mechanism-harness --lib semantics_relation_and_flag_oracles_match_observed_nodes -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_semantics_relations_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_combobox_active_descendant_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_roving_focus_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_focus_scope_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_nested_focus_scope_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_focus_scope_stale_parent_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness_recipe_layout_cases_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test focus_restore_mechanism_harness mechanism_harness_focus_restore_recipe_cases_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test recipe_typeahead_mechanism_harness mechanism_harness_recipe_typeahead_cases_match_oracles -- --nocapture
```

## View-Cache and Root-Boundary Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_layout_dirty_invalidation_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib view_cache -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib scroll_contained_view_cache_dirty_does_not_force_direct_child_root_invalidation -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib layout_request_build_roots_classify_view_cache_layout_dirty_expansion -- --nocapture
```

## Scroll-Handle Window-Update Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_scroll_handle_invalidation_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib view_cache_scroll -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib view_cache_virtual_list -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib retained_virtual_list_host_updates_window_without_rerendering_view_cache_root -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib scroll_handle_changes_classify -- --nocapture
```

## Environment View-Cache Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_environment_view_cache_invalidation_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib environment_ -- --nocapture
```

## Hit-Test Routing and Pointer Occlusion Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_hit_test_routing_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_pointer_occlusion_routing_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib pointer_occlusion -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib pointer_move_layers -- --nocapture
```

## Focus Barrier Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_focus_barrier_routing_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib focus_barrier -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib focus_scope -- --nocapture
```

## Semantics Relation Gates

```powershell
cargo test --profile dev-fast -p fret-mechanism-harness --lib semantics_relation_and_flag_oracles_match_observed_nodes -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_semantics_relations_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_combobox_active_descendant_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib text_input_semantics_controls_element_is_exposed -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib text_input_semantics_active_descendant_element_is_exposed -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib declarative_attach_semantics_can_override_state_and_relations -- --nocapture
```

## Roving Focus Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_roving_focus_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib roving_flex -- --nocapture
```

## Focus Scope Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_focus_scope_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_nested_focus_scope_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_focus_scope_stale_parent_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib focus_scope -- --nocapture
```

## Shadcn Focus Restore Recipe Gates

```powershell
cargo test --profile dev-fast -p fret-mechanism-harness --lib focus_oracle_can_match_restored_focus_outside_pointer_barrier -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test focus_restore_mechanism_harness mechanism_harness_focus_restore_recipe_cases_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test dialog_escape_dismiss_focus_restore dialog_escape_closes_and_restores_focus_to_trigger -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test popover_escape_dismiss_focus_restore popover_escape_closes_and_restores_focus_to_trigger -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test combobox_escape_dismiss_focus_restore combobox_escape_closes_and_restores_focus_to_trigger -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test select_escape_dismiss_focus_restore select_escape_closes_and_restores_focus_to_trigger -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test dropdown_menu_escape_dismiss_focus_restore dropdown_menu_escape_closes_and_restores_focus_to_trigger -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test context_menu_escape_dismiss_focus_clears context_menu_escape_closes_and_clears_focus -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test dialog_overlay_click_dismiss_focus_restore dialog_overlay_click_closes_and_restores_focus_to_trigger -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test popover_outside_click_dismiss_focus_restore popover_outside_click_closes_and_activates_underlay -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --lib context_menu_click_through_outside_press_closes_and_focuses_underlay -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --lib dropdown_menu_non_modal_outside_press_closes_without_restoring_focus_to_trigger -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --lib select_open_before_first_layout_installs_modal_barrier_and_blocks_underlay -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --lib popover_outside_press_can_be_intercepted -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --lib select_modal_barrier_dismiss_can_be_prevented_via_dismiss_handler -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --lib dropdown_menu_modal_outside_press_can_be_prevented_via_dismiss_handler -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --lib context_menu_click_through_outside_press_can_be_prevented_and_still_activates_underlay -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --lib context_menu_submenu_keyboard_open_transfers_focus_and_arrow_left_restores_focus -- --nocapture
cargo test --profile dev-fast -p fret-ui-kit --lib close_auto_focus_decision_maps_reasons -- --nocapture
cargo test --profile dev-fast -p fret-ui-kit --lib mouse_open_guard_pointer_up_decision_is_reusable_within_tick -- --nocapture
```

## Shadcn Recipe Typeahead Gates

```powershell
cargo test --profile dev-fast -p fret-ui-shadcn --test recipe_typeahead_mechanism_harness mechanism_harness_recipe_typeahead_cases_match_oracles -- --nocapture
```

## Shadcn Combobox Placement and Visual Gates

```powershell
cargo test -p fret-ui-shadcn --test web_vs_fret_overlay_placement web_vs_fret_combobox_cases_match_web_fixtures -- --nocapture
cargo test -p fret-ui-shadcn --lib combobox_trigger_places_chevron_at_inline_end -- --nocapture
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-demo-open-neutral-dark-screenshot.json --dir target/fret-diag-mechanism-harness-runtime --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-demo-narrow-open-screenshot.json --dir target/fret-diag-mechanism-harness-runtime --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
```

Current evidence anchors:

- Normal-width open screenshot after the chevron fix:
  `target/fret-diag-mechanism-harness-runtime/sessions/1778558055240-49296/screenshots/1778558059695-ui-gallery-combobox-basic-neutral-dark-open/window-4294967297-tick-101-frame-101.png`
- Normal-width open placement trace after the Popover size-hint fix:
  `target/fret-diag-combobox-check2/sessions/1778566847049-70708/1778566849917/script.result.json`
  - Trace: `desired.h_px=204.0`, `chosen_side=bottom`,
    `preferred_fits_without_main_clamp=true`.
- Normal-width open screenshot after the placement trace gate:
  `target/fret-diag-combobox-check2/sessions/1778566847049-70708/screenshots/1778566853802-ui-gallery-combobox-basic-neutral-dark-open/window-4294967297-tick-96-frame-96.png`
- Narrow-width open screenshot:
  `target/fret-diag-mechanism-harness-runtime/sessions/1778557035697-61384/screenshots/1778557037892-ui-gallery-combobox-demo-open-narrow/window-4294967297-tick-59-frame-59.png`
- Screenshot script hardening:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-demo-open-neutral-dark-screenshot.json`
- Combobox placement fixture suite:
  `ecosystem/fret-ui-shadcn/tests/fixtures/overlay_placement_combobox_cases_v1.json`

## Shadcn DropdownMenu Submenu Placement Gate

```powershell
cargo test -p fret-ui-shadcn --lib submenu_geometry_side_tracks_floating_position -- --nocapture
cargo test -p fret-ui-shadcn --lib dropdown_menu_submenu_keyboard_open_transfers_focus_and_arrow_left_restores_focus -- --nocapture
cargo check -p fret-ui-shadcn
cargo build -p fret-ui-gallery
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-submenu-open-smoke.json --dir target/fret-diag-dropdown-submenu-trace2 --session-auto --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
```

Current evidence anchors:

- Runtime submenu placement trace after adding DropdownMenu submenu diagnostics:
  `target/fret-diag-dropdown-submenu-trace2/sessions/1778568285442-34920/1778568287793/script.result.json`
  - Trace: `ui-gallery-dropdown-menu-submenu-invite-users` side `right`,
    `ui-gallery-dropdown-menu-submenu-more-options` side `right`.
- Layout sidecar:
  `target/fret-diag-dropdown-submenu-trace2/sessions/1778568285442-34920/1778568290949-ui-gallery-dropdown-menu-submenu-open-smoke.layout/layout.taffy.v1.json`
- Gate script:
  `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-submenu-open-smoke.json`
- Suite membership:
  `tools/diag-scripts/suites/ui-gallery/suite.json` and
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`

## Shadcn ContextMenu Submenu Placement Gate

```powershell
cargo test -p fret-ui-shadcn --lib submenu_geometry_side_tracks_floating_position -- --nocapture
cargo test -p fret-ui-shadcn --lib context_menu_submenu_keyboard_open_transfers_focus_and_arrow_left_restores_focus -- --nocapture
cargo build -p fret-ui-gallery
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-safe-corridor-sweep.json --dir target/fret-diag-context-menu-submenu-trace --session-auto --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
```

Current evidence anchors:

- Runtime submenu placement trace after adding the shared submenu diagnostics bridge to ContextMenu:
  `target/fret-diag-context-menu-submenu-trace/sessions/1778571251704-73756/1778571253975/script.result.json`
  - Trace: root content `chosen_side=right`; submenu anchor
    `ui-gallery-context-menu-submenu-more-tools` side `right`.
- Layout sidecar:
  `target/fret-diag-context-menu-submenu-trace/sessions/1778571251704-73756/1778571256920-ui-gallery-context-menu-submenu-safe-corridor-sweep.layout/layout.taffy.v1.json`
- Gate script:
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-safe-corridor-sweep.json`
- Suite membership:
  `tools/diag-scripts/suites/ui-gallery/suite.json` and
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`

## Runtime Diagnostics Gate

```powershell
$env:FRET_UI_GALLERY_VIEW_CACHE='1'
$env:FRET_UI_GALLERY_VIEW_CACHE_SHELL='1'
$env:FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE='1'
$env:FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE_PANIC='1'
target/debug/fretboard-dev.exe diag run ui-gallery-checkbox-demo-with-title-toggle-underflow --dir target/fret-diag/mechanism-harness-v1-checkbox-underflow --session-auto --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
```

Suite membership:

- `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-demo-with-title-toggle-underflow.json`
- `tools/diag-scripts/suites/diag-hardening-smoke/suite.json`

## UI Gallery Overlay/Focus Runtime Gate

```powershell
cargo build -p fret-ui-gallery --release
cargo run -p fretboard-dev -- diag suite fret-mechanism-harness-overlay-focus --dir target/fret-diag-mechanism-harness-runtime --session-auto --launch -- target/release/fret-ui-gallery.exe
```

Suite membership:

- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-focus-trap-tab-cycle.json`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-modal-barrier-focus-restore.json`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-detached-trigger-focus-restore.json`
- `tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-outside-press-focus-restore.json`
- `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-dismiss-outside-press.json`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-escape-focus-restore.json`
- `tools/diag-scripts/suites/fret-mechanism-harness-overlay-focus/suite.json`

Fast local rerun after the release binary already exists:

```powershell
cargo run -p fretboard-dev -- diag suite fret-mechanism-harness-overlay-focus --dir target/fret-diag-mechanism-harness-runtime --session-auto --launch -- target/release/fret-ui-gallery.exe
```

## Repo Integrity Gates

```powershell
python -m json.tool docs/workstreams/fret-mechanism-harness-v1/WORKSTREAM.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/layout_dirty_invalidation_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/scroll_handle_invalidation_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/environment_view_cache_invalidation_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/semantics_relations_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/combobox_active_descendant_interaction_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/roving_focus_interaction_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/focus_scope_interaction_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/focus_scope_nested_interaction_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/focus_scope_stale_parent_interaction_v1.json | Out-Null
python -m json.tool ecosystem/fret-ui-shadcn/tests/fixtures/focus_restore_recipe_cases_v1.json | Out-Null
python -m json.tool ecosystem/fret-ui-shadcn/tests/fixtures/recipe_typeahead_cases_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/pointer_occlusion_routing_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/focus_barrier_routing_v1.json | Out-Null
python tools/check_workstream_catalog.py
python tools/check_diag_scripts_registry.py
cargo fmt --package fret-mechanism-harness --package fret-ui --package fret-ui-shadcn --check
```

## Evidence Anchors

- Harness architecture: `docs/mechanism-harness-v2.md`
- Scalar metrics: `crates/fret-mechanism-harness/src/observe.rs`,
  `crates/fret-mechanism-harness/src/oracle.rs`
- Layout dirty fixture: `crates/fret-ui/src/tree/tests/fixtures/layout_dirty_invalidation_v1.json`
- Layout dirty runner: `crates/fret-ui/src/tree/tests/layout_dirty_invalidation_harness.rs`
- Scroll-handle fixture: `crates/fret-ui/src/tree/tests/fixtures/scroll_handle_invalidation_v1.json`
- Scroll-handle runner: `crates/fret-ui/src/tree/tests/scroll_handle_invalidation_harness.rs`
- Environment view-cache fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/environment_view_cache_invalidation_v1.json`
- Environment view-cache runner:
  `crates/fret-ui/src/declarative/tests/environment_view_cache_harness.rs`
- Hit-test routing fixture: `crates/fret-ui/src/declarative/tests/fixtures/hit_test_routing_v1.json`
- Hit-test routing runner: `crates/fret-ui/src/declarative/tests/layout/hit_test_mechanism_harness.rs`
- Pointer occlusion routing fixture:
  `crates/fret-ui/src/tree/tests/fixtures/pointer_occlusion_routing_v1.json`
- Pointer occlusion routing runner:
  `crates/fret-ui/src/tree/tests/pointer_occlusion_routing_harness.rs`
- Focus barrier routing fixture:
  `crates/fret-ui/src/tree/tests/fixtures/focus_barrier_routing_v1.json`
- Focus barrier routing runner:
  `crates/fret-ui/src/tree/tests/focus_barrier_routing_harness.rs`
- Semantics relation fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/semantics_relations_v1.json`
- Semantics relation runner:
  `crates/fret-ui/src/declarative/tests/semantics_relations_harness.rs`
- Combobox active-descendant interaction fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/combobox_active_descendant_interaction_v1.json`
- Combobox active-descendant interaction runner:
  `crates/fret-ui/src/declarative/tests/combobox_active_descendant_interaction_harness.rs`
- Roving focus/typeahead interaction fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/roving_focus_interaction_v1.json`
- Roving focus/typeahead interaction runner:
  `crates/fret-ui/src/declarative/tests/roving_focus_interaction_harness.rs`
- Focus scope interaction fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/focus_scope_interaction_v1.json`
- Focus scope interaction runner:
  `crates/fret-ui/src/declarative/tests/focus_scope_interaction_harness.rs`
- Nested focus scope interaction fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/focus_scope_nested_interaction_v1.json`
- Nested focus scope interaction runner:
  `crates/fret-ui/src/declarative/tests/focus_scope_interaction_harness.rs`
- Stale-parent focus scope fixture:
  `crates/fret-ui/src/tree/tests/fixtures/focus_scope_stale_parent_interaction_v1.json`
- Stale-parent focus scope runner:
  `crates/fret-ui/src/tree/tests/focus_scope_stale_parent_harness.rs`
- Shadcn focus restore recipe fixture:
  `ecosystem/fret-ui-shadcn/tests/fixtures/focus_restore_recipe_cases_v1.json`
  - now includes dropdown-menu, context-menu, and menubar submenu keyboard open / ArrowLeft
    restore cases with `submenu.opened` and `submenu.closed` metrics
- Shadcn focus restore recipe runner:
  `ecosystem/fret-ui-shadcn/tests/focus_restore_mechanism_harness.rs`
- Shadcn recipe typeahead fixture:
  `ecosystem/fret-ui-shadcn/tests/fixtures/recipe_typeahead_cases_v1.json`
- Shadcn recipe typeahead runner:
  `ecosystem/fret-ui-shadcn/tests/recipe_typeahead_mechanism_harness.rs`
- UI Gallery overlay/focus runtime suite:
  `tools/diag-scripts/suites/fret-mechanism-harness-overlay-focus/suite.json`
- UI Gallery overlay/focus runtime scripts:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-focus-trap-tab-cycle.json`,
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-modal-barrier-focus-restore.json`,
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-detached-trigger-focus-restore.json`,
  `tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-outside-press-focus-restore.json`,
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-dismiss-outside-press.json`,
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-escape-focus-restore.json`
- Combobox popup-trigger placement gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-trigger.json`
  - asserts collision flip to top and `side_offset_px=6`
  - evidence:
    `target/fret-diag-combobox-popup-position-side-offset/sessions/1778576696581-76876/1778576699867/script.result.json`
  - layout sidecar:
    `target/fret-diag-combobox-popup-position-side-offset/sessions/1778576696581-76876/1778576702191-ui-gallery-combobox-popup-trigger-open.layout/layout.taffy.v1.json`
  - screenshot:
    `target/fret-diag-combobox-popup-position-side-offset/sessions/1778576696581-76876/screenshots/1778576702321-ui-gallery-combobox-popup-trigger-open/window-4294967297-tick-83-frame-83.png`
- Companion Combobox popup-trigger bottom-room gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-trigger-bottom-room.json`
  - asserts preferred-bottom placement with `chosen_side=bottom`, `flipped=false`, and `side_offset_px=6`
  - evidence:
    `target/fret-diag-combobox-popup-bottom-room/sessions/1778578242074-69792/1778578244187/script.result.json`
  - layout sidecar:
    `target/fret-diag-combobox-popup-bottom-room/sessions/1778578242074-69792/1778578245269-ui-gallery-combobox-popup-trigger-bottom-room-open.layout/layout.taffy.v1.json`
  - screenshot:
    `target/fret-diag-combobox-popup-bottom-room/sessions/1778578242074-69792/screenshots/1778578245323-ui-gallery-combobox-popup-trigger-bottom-room-open/window-4294967297-tick-62-frame-62.png`
- Menubar submenu placement gates:
  `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-submenu-placement-trace.json`,
  `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-rtl-submenu-placement-trace.json`
  - LTR asserts the Demo `Share` submenu placed-rect trace opens to physical right.
  - RTL asserts the RTL `Share` submenu placed-rect trace opens to physical left in a wide
    viewport where the preferred inline-end side has enough room.
  - LTR evidence:
    `target/fret-diag-menubar-submenu-placement-ltr-final/sessions/1778580950360-48828/1778580953153-ui-gallery-menubar-submenu-placement-trace/script.result.json`
  - LTR layout sidecar:
    `target/fret-diag-menubar-submenu-placement-ltr-final/sessions/1778580950360-48828/1778580953105-ui-gallery-menubar-submenu-placement-trace.layout/layout.taffy.v1.json`
  - RTL evidence:
    `target/fret-diag-menubar-rtl-submenu-placement-wide/sessions/1778580931311-68804/1778580934199-ui-gallery-menubar-rtl-submenu-placement-trace/script.result.json`
  - RTL layout sidecar:
    `target/fret-diag-menubar-rtl-submenu-placement-wide/sessions/1778580931311-68804/1778580934010-ui-gallery-menubar-rtl-submenu-placement-trace.layout/layout.taffy.v1.json`
  - RTL screenshot:
    `target/fret-diag-menubar-rtl-submenu-placement-wide/sessions/1778580931311-68804/screenshots/1778580934055-ui-gallery-menubar-rtl-submenu-placement-trace/window-4294967297-tick-40-frame-40.png`
- Text render instance binding fix:
  `crates/fret-render-wgpu/src/renderer/render_scene/recorders/scene_draw.rs`,
  `crates/fret-render-wgpu/src/renderer/pipelines/text.rs`
- Previous focused tests: `crates/fret-ui/src/tree/tests/subtree_layout_dirty_underflow_repair.rs`
- View-cache focused tests: `crates/fret-ui/src/tree/tests/view_cache.rs`
- Environment focused tests: `crates/fret-ui/src/declarative/tests/environment_queries.rs`
- Pointer occlusion focused tests: `crates/fret-ui/src/tree/tests/pointer_occlusion.rs`
- Pointer move/capture focused tests: `crates/fret-ui/src/tree/tests/pointer_move_layers.rs`
- Focus barrier focused tests: `crates/fret-ui/src/tree/tests/focus_barrier_transition.rs`
- Focus scope focused tests: `crates/fret-ui/src/tree/tests/focus_scope.rs`,
  `crates/fret-ui/src/tree/tests/focus_scope_layered.rs`
- Semantics relation focused tests: `crates/fret-ui/src/declarative/tests/interactions/text_input.rs`,
  `crates/fret-ui/src/declarative/tests/semantics.rs`
- Roving focus focused tests: `crates/fret-ui/src/declarative/tests/interactions/roving_flex.rs`
- Shadcn focus restore focused tests:
  `ecosystem/fret-ui-shadcn/tests/dialog_escape_dismiss_focus_restore.rs`,
  `ecosystem/fret-ui-shadcn/tests/popover_escape_dismiss_focus_restore.rs`,
  `ecosystem/fret-ui-shadcn/tests/combobox_escape_dismiss_focus_restore.rs`,
  `ecosystem/fret-ui-shadcn/tests/select_escape_dismiss_focus_restore.rs`,
  `ecosystem/fret-ui-shadcn/tests/dropdown_menu_escape_dismiss_focus_restore.rs`,
  `ecosystem/fret-ui-shadcn/tests/context_menu_escape_dismiss_focus_clears.rs`,
  `ecosystem/fret-ui-shadcn/tests/dialog_overlay_click_dismiss_focus_restore.rs`,
  `ecosystem/fret-ui-shadcn/tests/popover_outside_click_dismiss_focus_restore.rs`
- Shadcn/lib outside-press focused tests:
  `ecosystem/fret-ui-shadcn/src/popover.rs`,
  `ecosystem/fret-ui-shadcn/src/context_menu.rs`,
  `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`,
  `ecosystem/fret-ui-shadcn/src/select.rs`
- Shadcn submenu restore focused tests:
  `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`,
  `ecosystem/fret-ui-shadcn/src/context_menu.rs`,
  `ecosystem/fret-ui-shadcn/src/menubar.rs`
- Combobox reason policy focused tests:
  `ecosystem/fret-ui-kit/src/primitives/combobox.rs`
- Select mouse-open pointer-up guard focused tests:
  `ecosystem/fret-ui-kit/src/primitives/select.rs`
- Retained virtual-list focused test: `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`
- Scroll registry classification tests: `crates/fret-ui/src/declarative/frame.rs`
- Scroll-contained frontier focused test: `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
- Layout request attribution focused test:
  `crates/fret-ui/src/tree/tests/interactive_resize_flow_rebuild.rs`
- Runtime script:
  `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-demo-with-title-toggle-underflow.json`
