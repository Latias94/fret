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
cargo nextest run -p fret-ui mechanism_harness_anchored_panel_placement_matches_oracles
cargo nextest run -p fret-ui mechanism_harness_anchored_layout_invalidation_matches_oracles
cargo test --profile dev-fast -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness_recipe_layout_cases_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test focus_restore_mechanism_harness mechanism_harness_focus_restore_recipe_cases_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test recipe_typeahead_mechanism_harness mechanism_harness_recipe_typeahead_cases_match_oracles -- --nocapture
```

## Core Overlay Placement Gates

```powershell
python -m json.tool crates/fret-ui/src/overlay_placement/fixtures/anchored_panel_placement_v1.json
cargo nextest run -p fret-ui mechanism_harness_anchored_panel_placement_matches_oracles
```

## Anchored Layout Invalidation Gates

```powershell
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/anchored_layout_invalidation_v1.json
cargo nextest run -p fret-ui mechanism_harness_anchored_layout_invalidation_matches_oracles
cargo nextest run -p fret-ui anchored_anchor_element_uses_render_transformed_visual_bounds
cargo nextest run -p fret-ui anchored_anchor_element_uses_scroll_transformed_visual_bounds
cargo nextest run -p fret-ui anchored
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
cargo nextest run -p fret-ui mechanism_harness_retained_virtual_list_reconcile_matches_oracles
cargo nextest run -p fret-ui mechanism_harness_prepaint_virtual_list_window_update_matches_oracles
cargo test --profile dev-fast -p fret-ui --lib scroll_handle_changes_classify -- --nocapture
cargo test --profile dev-fast -p fret-diag-protocol --lib predicate_virtual_list_window_shift_samples_len_le_serializes -- --nocapture
cargo test --profile dev-fast -p fret-diag-protocol --lib step_assert_semantics_scroll_idle_stable_deserializes_with_defaults -- --nocapture
cargo test --profile dev-fast -p fret-bootstrap --features diagnostics,ui-app-driver --lib step_start_retains_semantics_scroll_idle_stable_trace_for_passed_evidence -- --nocapture
cargo build -p fret-ui-gallery --release
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-small-scroll-no-window-shifts.json --dir target/fret-diag-virtual-list-harness-commit-check --session-auto --pack --ai-packet --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard-dev -- diag suite ui-gallery-vlist-window-boundary --dir target/fret-diag-vlist-window-boundary-after-reason-fix --session-auto --launch -- cargo run -p fret-ui-gallery --features gallery-dev
cargo run -p fretboard-dev -- diag suite ui-gallery-vlist-window-boundary-retained --dir target/fret-diag-vlist-window-boundary-retained-bounce-script-final --session-auto --launch -- target/debug/fret-ui-gallery.exe
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-scroll-to-rtl-field.json --dir target/fret-diag-rtl-scroll-idle-stability-v2 --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-rtl-idle-stability.json --dir target/fret-diag-scroll-area-rtl-idle-stability --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/table/ui-gallery-table-rtl-idle-stability.json --dir target/fret-diag-table-rtl-idle-stability --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
```

Current runtime evidence anchors:

- Virtual-list small-scroll no-window-shift gate:
  `tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-small-scroll-no-window-shifts.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-vlist-no-window-shifts-small-scroll/suite.json`
  - evidence:
    `target/fret-diag-virtual-list-harness-commit-check/sessions/1778697640180-135472/1778697641371/script.result.json`
  - share pack:
    `target/fret-diag-virtual-list-harness-commit-check/sessions/1778697640180-135472/share/1778697641371.zip`
- Virtual-list boundary-crossing non-retained gate:
  `tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-window-boundary-scroll.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-vlist-window-boundary/suite.json`
  - evidence:
    `target/fret-diag-vlist-window-boundary-after-reason-fix/sessions/1778718360331-96416/suite.summary.json`
  - proof:
    suite `status=passed`.
- Virtual-list boundary-crossing retained gate:
  `tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-window-boundary-scroll-retained.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-vlist-window-boundary-retained/suite.json`
  - current evidence:
    `target/fret-diag-vlist-window-boundary-retained-bounce-script-final/sessions/1778726366803-140844/suite.summary.json`
  - status:
    suite `status=passed`; the script now bounces back after the boundary-crossing scroll and
    asserts retained keep-alive reuse before capture.
  - proof:
    bundle frame 54 recorded an `escape` retained reconcile with
    `reused_from_keep_alive_items=9` and `keep_alive_pool_len_after=9`.
- Synthetic retained-host reconcile fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/retained_virtual_list_reconcile_v1.json`
  - runner:
    `crates/fret-ui/src/declarative/tests/retained_virtual_list_reconcile_harness.rs`
  - proof:
    asserts the bounce scenario records keep-alive insertion on downward scroll and reuse on the
    return scroll while avoiding cache-root rerendering after warmup.
  - current command:
    `cargo nextest run -p fret-ui mechanism_harness_retained_virtual_list_reconcile_matches_oracles`
  - current result:
    passed, 1 test; Nextest run id `0ab07b84-dfed-4198-8a55-70754688b874`.
- Synthetic prepaint virtual-list window-update fixture:
  `crates/fret-ui/src/tree/prepaint/tests/fixtures/virtual_list_window_update_v1.json`
  - runner:
    `crates/fret-ui/src/tree/prepaint/tests/prepaint_virtual_list_window_update_harness.rs`
  - proof:
    asserts prepaint window-shift kind/reason/detail and dirty cache-root attribution for scroll
    offset, viewport resize, items revision, and scroll-to-item non-retained paths.
  - first failed evidence:
    Nextest run id `7f2de95b-4436-4bd7-ad2e-44e818e389b4`; viewport-resize and items-revision
    cases had the right prepaint debug detail but no matching cache-root dirty reason.
  - current command:
    `cargo nextest run -p fret-ui mechanism_harness_prepaint_virtual_list_window_update_matches_oracles`
  - current result:
    passed, 1 test; Nextest run id `7fd6de79-fd32-421c-88f4-7844cc05ea2f` after adding the
    length-shrink inputs-change case.
  - companion focused result:
    `cargo nextest run -p fret-ui prepaint_detects_render_window_insufficient_for_overscan_policy prepaint_marks_scroll_to_item_window_updates_with_distinct_invalidation_detail prepaint_attributes_window_escape_to_scroll_offset_when_state_offset_was_synced prepaint_updates_virtual_list_window_and_marks_cache_root_dirty_on_escape virtual_list_window_shift_detail_classifies_items_revision view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
    passed, 6 tests; Nextest run id `3f91c35e-f1ff-4e06-8b13-155dc289a493`.
- Checkbox RTL post-scroll idle-stability gate:
  `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-scroll-to-rtl-field.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-rtl-smoke/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - evidence:
    `target/fret-diag-rtl-scroll-idle-stability-v2/sessions/1778699811444-47076/1778699812656/script.result.json`
  - share pack:
    `target/fret-diag-rtl-scroll-idle-stability-v2/sessions/1778699811444-47076/share/1778699812656.zip`
  - trace proof:
    `sample_count=45`, `required_samples=45`, `baseline_value=2495.999755859375`,
    `value=2495.999755859375`, `frame_delta=0.0`, `total_delta=0.0`.
- ScrollArea RTL nested-scroll idle-stability gate:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-rtl-idle-stability.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-rtl-smoke/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - evidence:
    `target/fret-diag-scroll-area-rtl-idle-stability/sessions/1778705030563-134220/1778705031873/script.result.json`
  - share pack:
    `target/fret-diag-scroll-area-rtl-idle-stability/sessions/1778705030563-134220/share/1778705031873.zip`
  - trace proof:
    nested RTL viewport `sample_count=60`, `baseline_value=480.0`, `value=480.0`,
    `frame_delta=0.0`, `total_delta=0.0`; outer content viewport `sample_count=60`,
    `baseline_value=1220.0`, `value=1220.0`, `frame_delta=0.0`, `total_delta=0.0`.
- Table RTL post-scroll idle-stability gate:
  `tools/diag-scripts/ui-gallery/table/ui-gallery-table-rtl-idle-stability.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-rtl-smoke/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - evidence:
    `target/fret-diag-table-rtl-idle-stability/sessions/1778705380996-128024/1778705382221/script.result.json`
  - share pack:
    `target/fret-diag-table-rtl-idle-stability/sessions/1778705380996-128024/share/1778705382221.zip`
  - trace proof:
    outer content viewport `sample_count=60`, `baseline_value=1260.0`, `value=1260.0`,
    `frame_delta=0.0`, `total_delta=0.0`.

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
cargo test --profile dev-fast -p fret-mechanism-harness --lib default_selectors_exclude_semantics_hidden_subtrees_but_flags_remain_queryable -- --nocapture
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
cargo test -p fret-ui-shadcn --lib combobox_trigger_long_label_stays_before_chevron -- --nocapture
cargo test -p fret-ui-shadcn --lib command_palette_test_id_prefix_derives_surface_ids -- --nocapture
cargo nextest run -p fret-ui-shadcn popover_first_open_center_alignment_uses_explicit_width_for_x
cargo test -p fret-ui-shadcn --lib popover::tests::popover_transformed_trigger_uses_visual_anchor_bounds -- --exact --nocapture
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-demo-open-neutral-dark-screenshot.json --dir target/fret-diag-mechanism-harness-runtime --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-demo-narrow-open-screenshot.json --dir target/fret-diag-mechanism-harness-runtime --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-open.json --dir target/fret-diag-cb-responsive-tightened --session-auto --launch -- target/release/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-trigger.json --dir target/fret-diag-combobox-popup-label-checkmark --session-auto --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-trigger-bottom-room.json --dir target/fret-diag-combobox-popup-bottom-room-label-checkmark --session-auto --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-long-text-geometry.json --dir target/fret-diag-combobox-long-text-geometry-v4 --session-auto --launch -- target/debug/fret-ui-gallery.exe
```

Current evidence anchors:

- Visual parity matrix:
  `docs/workstreams/fret-mechanism-harness-v1/VISUAL_PARITY_MATRIX.md`
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
- Trigger chrome screenshot gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-trigger-screenshot-zinc-dark.json`
  - scrolls `ui-gallery-nav-combobox` into view before `click_stable`
  - asserts `ui-gallery-combobox-multiple-trigger` stays within the window and at least
    `240px x 32px`
  - captures a layout sidecar for the trigger chrome section
  - suite membership: `tools/diag-scripts/suites/ui-gallery-combobox/suite.json`
  - evidence:
    `target/fret-diag-combobox-trigger-zinc-dark/sessions/1778609257776-96708/1778609258840/script.result.json`
  - screenshot:
    `target/fret-diag-combobox-trigger-zinc-dark/sessions/1778609257776-96708/screenshots/.../window-4294967297-tick-75-frame-75.png`
- Combobox placement fixture suite:
  `ecosystem/fret-ui-shadcn/tests/fixtures/overlay_placement_combobox_cases_v1.json`
- Responsive Combobox desktop placement gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-open.json`
  - asserts `preferred_side=bottom`, `chosen_side=bottom`, `flipped=false`, and `side_offset_px=6`
  - asserts visible content left aligns with the trigger and visible content width is 200px against
    the 150px responsive trigger
  - suite membership: `tools/diag-scripts/suites/ui-gallery-combobox/suite.json`
  - evidence:
    `target/fret-diag-cb-responsive-tightened/sessions/1778583319711-74988/script.result.json`
  - trace: `anchor=ui-gallery-combobox-responsive-trigger`,
    `content=ui-gallery-combobox-responsive-content`, `chosen_side=bottom`, `side_offset_px=6`
- Command item internal-anchor gate:
  `ecosystem/fret-ui-shadcn/src/command.rs`
  - test: `command_palette_test_id_prefix_derives_surface_ids`
  - protects `.chrome`, `.label`, and `.checkmark` surfaces derived from a `test_id_prefix` so
    runtime diagnostics can inspect item internals.
- Popup-trigger label/checkmark geometry gates:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-trigger.json` and
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-trigger-bottom-room.json`
  - assert listbox/option size, row spacing, label size, label vertical centering, label left inset,
    checkmark size, checkmark vertical centering, and checkmark left inset.
  - top-flip evidence:
    `target/fret-diag-combobox-popup-label-checkmark/sessions/1778616773537-101944/script.result.json`
  - top-flip bundle with item child anchors:
    `target/fret-diag-combobox-popup-label-checkmark/sessions/1778616773537-101944/1778616779787-ui-gallery-combobox-popup-trigger-open/bundle.schema2.json`
  - bottom-room evidence:
    `target/fret-diag-combobox-popup-bottom-room-label-checkmark/sessions/1778616793292-16876/script.result.json`
  - bottom-room bundle with item child anchors:
    `target/fret-diag-combobox-popup-bottom-room-label-checkmark/sessions/1778616793292-16876/1778616796767-ui-gallery-combobox-popup-trigger-bottom-room-open/bundle.schema2.json`
- Combobox long-text trigger/option geometry gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-long-text-geometry.json`
  - asserts trigger label width budget, label-before-chevron right delta, chrome-relative vertical
    centering, popup placement, option label width budget, option label/checkmark insets, and
    option label vertical centering.
  - focused test:
    `combobox_trigger_long_label_stays_before_chevron`
  - evidence:
    `target/fret-diag-combobox-long-text-geometry-v4/sessions/1778619498565-104108/script.result.json`
  - bundle with long-text child anchors:
    `target/fret-diag-combobox-long-text-geometry-v4/sessions/1778619498565-104108/1778619501160-ui-gallery-combobox-long-text-open/bundle.schema2.json`
- Popover first-open explicit-width center alignment gate:
  `ecosystem/fret-ui-shadcn/src/popover.rs`
  - test: `popover_first_open_center_alignment_uses_explicit_width_for_x`
  - protects the component bridge from reusing the default `288px` estimate for visible x placement
    when explicit content width is smaller.
- Popover transformed-trigger visual anchor gate:
  `ecosystem/fret-ui-shadcn/src/popover.rs`
  - test: `popover_transformed_trigger_uses_visual_anchor_bounds`
  - protects the recipe path from anchoring to wrapper/layout x=`40` instead of the inner
    transformed trigger x=`340`.

## Shadcn Button Group Layout Gates

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/button/ui-gallery-button-group-size-screenshots-zinc-light-dark.json --dir target/fret-diag-button-group-size --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/control-chrome/ui-gallery-control-chrome-button-group-text-w-fit.json --dir target/fret-diag-control-chrome-button-group-text --session-auto --pack --ai-packet --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-geometry.json --dir target/fret-diag-button-group-input-group-geometry --session-auto --pack --ai-packet --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-long-text.json --dir target/fret-diag-button-group-input-group-long-text --session-auto --pack --ai-packet --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-rtl-addon-order.json --dir target/fret-diag-input-group-rtl-addon-order --session-auto --pack --ai-packet --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/input/ui-gallery-input-button-group-and-file-controls-fill.json --dir target/fret-diag-input-button-group-fill --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
cargo test -p fret-ui-shadcn --lib button_group_text_derives_internal_label_test_id -- --nocapture
cargo test -p fret-ui-shadcn --lib button_group_text_custom_children_do_not_get_derived_label_test_id -- --nocapture
cargo test -p fret-ui-shadcn --lib input_group_button_derives_internal_test_ids -- --nocapture
cargo test -p fret-ui-shadcn --lib input_group_text_stamps_test_id -- --nocapture
cargo test -p fret-diag-protocol --lib predicate_text_input_ime_cursor_area_within_bounds_serializes_and_deserializes -- --nocapture
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/control-chrome/ui-gallery-control-chrome-button-group-text-w-fit.json --dir target/fret-diag-button-group-text-alignment-v2 --session-auto --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-geometry.json --dir target/fret-diag-button-group-input-group-geometry-final-current --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-long-text.json --dir target/fret-diag-button-group-input-group-long-text-final-current --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-rtl-addon-order.json --dir target/fret-diag-input-group-rtl-addon-order-final-current --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
```

Current evidence anchors:

- Size gate evidence:
  `target/fret-diag-button-group-size/...`
- Size layout sidecar:
  `target/fret-diag-button-group-size/.../layout.taffy.v1.json`
- The related control-chrome and input gates are tracked in the same shadcn suite:
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
- ButtonGroupText alignment gate:
  `tools/diag-scripts/ui-gallery/control-chrome/ui-gallery-control-chrome-button-group-text-w-fit.json`
  - asserts `w-fit`, prefix/suffix label size, prefix/suffix label vertical centering inside their
    segments, and prefix/suffix segment vertical centering against the input control.
  - focused internal-anchor tests:
    `button_group_text_derives_internal_label_test_id` and
    `button_group_text_custom_children_do_not_get_derived_label_test_id`.
  - evidence:
    `target/fret-diag-button-group-text-alignment-v2/sessions/1778621202064-80960/script.result.json`
  - bundle with label anchors:
    `target/fret-diag-button-group-text-alignment-v2/sessions/1778621202064-80960/1778621207864-ui-gallery-control-chrome-button-group-text-w-fit/bundle.schema2.json`
- Button Group Input Group geometry gate:
  `tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-geometry.json`
  - asserts root/control/add-button/voice-button/icon sizes, voice icon centering inside the voice
    button, voice button vertical centering against the input, and input-vs-trailing-button
    non-overlap.
  - focused internal-anchor test:
    `input_group_button_derives_internal_test_ids`.
  - suite redirect:
    `tools/diag-scripts/ui-gallery-button-group-input-group-geometry.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-button-group-input-group-geometry`)
  - current evidence:
    `target/fret-diag-button-group-input-group-geometry-final-current/1778624775617/script.result.json`
  - current share pack:
    `target/fret-diag-button-group-input-group-geometry-final-current/share/1778624775617.zip`
- Button Group Input Group long-text caret/bounds gate:
  `tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-long-text.json`
  - injects a long value, asserts the value remains present, the input/root stay bounded, the
    trailing voice button does not overlap the input, and the runtime IME/caret area stays within
    the input control bounds.
  - mechanism predicate contract:
    `text_input_ime_cursor_area_within_bounds`.
  - protocol roundtrip test:
    `predicate_text_input_ime_cursor_area_within_bounds_serializes_and_deserializes`.
  - suite redirect:
    `tools/diag-scripts/ui-gallery-button-group-input-group-long-text.json`
  - current evidence:
    `target/fret-diag-button-group-input-group-long-text-final-current/1778626121814/script.result.json`
  - current share pack:
    `target/fret-diag-button-group-input-group-long-text-final-current/share/1778626121814.zip`
- Input Group RTL addon-order gate:
  `tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-rtl-addon-order.json`
  - asserts logical leading/trailing addons map to the correct physical sides under RTL, remain
    non-overlapping with the control, and stay vertically centered against the control.
  - focused internal-anchor test:
    `input_group_text_stamps_test_id`.
  - current evidence:
    `target/fret-diag-input-group-rtl-addon-order-final-current/1778627682652/script.result.json`
  - current share pack:
    `target/fret-diag-input-group-rtl-addon-order-final-current/share/1778627682652.zip`

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
- Hidden-subtree selector/oracle fix:
  `crates/fret-mechanism-harness/src/observe.rs`,
  `crates/fret-mechanism-harness/src/oracle.rs`
- Hidden-subtree focused gate:
  `default_selectors_exclude_semantics_hidden_subtrees_but_flags_remain_queryable`
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
- Anchored panel overlay placement fixture:
  `crates/fret-ui/src/overlay_placement/fixtures/anchored_panel_placement_v1.json`
- Anchored panel overlay placement runner:
  `crates/fret-ui/src/overlay_placement/tests.rs`
- Anchored layout invalidation fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/anchored_layout_invalidation_v1.json`
- Anchored layout invalidation runner:
  `crates/fret-ui/src/declarative/tests/anchored_layout_invalidation_harness.rs`
- Anchored prop-diff fix:
  `crates/fret-ui/src/declarative/mount.rs`
- Anchored transformed-anchor fix:
  `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - focused gate: `anchored_anchor_element_uses_render_transformed_visual_bounds`
  - before fix: panel x=`0` from raw layout bounds; after fix: panel x=`40` from visual bounds.
- Anchored scroll-transformed anchor fix:
  `crates/fret-ui/src/tree/layout/state.rs`,
  `crates/fret-ui/src/tree/ui_tree_debug/query.rs`
  - focused gate: `anchored_anchor_element_uses_scroll_transformed_visual_bounds`
  - before fix: panel y=`90` from stale/content-space placement; after fix: panel y=`30` from
    scrolled visual bounds.
  - companion scroll gates:
    `scroll_handle_set_offset_triggers_visual_scroll_without_manual_invalidate` and
    `scroll_wheel_updates_offset_and_shifts_child_bounds`
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
  - asserts collision flip to top, `side_offset_px=6`, visible and stable trigger/content-shell
    bounds, content-shell top/bottom side gap, content left/width alignment with the trigger,
    listbox min/max size, option min/max size, and first-to-second row spacing
  - current content-shell evidence:
    `target/fret-diag-combobox-popup-trigger-content-current/1778629033429/script.result.json`
  - current content-shell share pack:
    `target/fret-diag-combobox-popup-trigger-content-current/share/1778629033429.zip`
  - bounds proof:
    content bottom `369.33331`, trigger top `375.3333`, gap `6px`
  - current deterministic tight-window evidence:
    `target/fret-diag-combobox-popup-tight-fixed-current/script.result.json`
  - current share pack:
    `target/fret-diag-combobox-popup-tight-fixed-current/share/1778622970638.zip`
  - current Codex rerun evidence:
    `target/fret-diag/codex-combobox-popup-tight/sessions/1778634941245-107020/script.result.json`
  - current Codex rerun share pack:
    `target/fret-diag/codex-combobox-popup-tight/sessions/1778634941245-107020/share/1778634943439.zip`
  - current Codex trace proof:
    preferred `Bottom`, chosen `Top`, flipped, side offset `6.0`, final rect `(377.7, 165.3, 256.0, 204.0)`,
    shift `(0.0, 0.0)`
  - current Codex screenshot:
    `target/fret-diag/codex-combobox-popup-tight/sessions/1778634941245-107020/screenshots/1778634944932-ui-gallery-combobox-popup-trigger-open/window-4294967297-tick-79-frame-79.png`
  - evidence:
    `target/fret-diag-combobox-popup-trigger-visible-bounds/sessions/1778609728199-76436/1778609729283/script.result.json`
  - row-bounds evidence:
    `target/fret-diag-combobox-popup-trigger-row-bounds/sessions/1778610794278-89396/1778610796686/script.result.json`
  - layout sidecar:
    `target/fret-diag-combobox-popup-position-side-offset/sessions/1778576696581-76876/1778576702191-ui-gallery-combobox-popup-trigger-open.layout/layout.taffy.v1.json`
  - screenshot:
    `target/fret-diag-combobox-popup-position-side-offset/sessions/1778576696581-76876/screenshots/1778576702321-ui-gallery-combobox-popup-trigger-open/window-4294967297-tick-83-frame-83.png`
- Companion Combobox popup-trigger bottom-room gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-trigger-bottom-room.json`
  - asserts preferred-bottom placement with `chosen_side=bottom`, `flipped=false`, and `side_offset_px=6`
  - asserts visible and stable trigger/content-shell bounds, content-shell top/bottom side gap,
    content left/width alignment with the trigger, listbox min/max size, option min/max size, and
    first-to-second row spacing
  - current content-shell evidence:
    `target/fret-diag-combobox-popup-bottom-room-content-current/1778629019743/script.result.json`
  - current content-shell share pack:
    `target/fret-diag-combobox-popup-bottom-room-content-current/share/1778629019743.zip`
  - bounds proof:
    trigger bottom `503.3333`, content top `509.33334`, gap `6px`
  - current debug-exe evidence:
    `target/fret-diag-combobox-popup-bottom-room-debug-current/script.result.json`
  - current share pack:
    `target/fret-diag-combobox-popup-bottom-room-debug-current/share/1778622441910.zip`
  - evidence:
    `target/fret-diag-combobox-popup-bottom-room-visible-bounds/sessions/1778609769648-73816/1778609770731/script.result.json`
  - row-bounds evidence:
    `target/fret-diag-combobox-popup-bottom-room-row-bounds/sessions/1778610812280-94832/1778610814288/script.result.json`
  - layout sidecar:
    `target/fret-diag-combobox-popup-bottom-room/sessions/1778578242074-69792/1778578245269-ui-gallery-combobox-popup-trigger-bottom-room-open.layout/layout.taffy.v1.json`
  - screenshot:
    `target/fret-diag-combobox-popup-bottom-room/sessions/1778578242074-69792/screenshots/1778578245323-ui-gallery-combobox-popup-trigger-bottom-room-open/window-4294967297-tick-62-frame-62.png`
  - current Codex rerun evidence:
    `target/fret-diag/codex-combobox-popup-bottom/sessions/1778634652696-96372/script.result.json`
  - current Codex rerun share pack:
    `target/fret-diag/codex-combobox-popup-bottom/sessions/1778634652696-96372/share/1778634922898.zip`
  - current Codex trace proof:
    preferred `Bottom`, chosen `Bottom`, side offset `6.0`, final rect `(377.7, 509.3, 256.0, 204.0)`,
    shift `(0.0, 0.0)`
  - current Codex screenshot:
    `target/fret-diag/codex-combobox-popup-bottom/sessions/1778634652696-96372/screenshots/1778634924219-ui-gallery-combobox-popup-trigger-bottom-room-open/window-4294967297-tick-69-frame-69.png`
- Button Group Input Group long-text visible-text gate:
  `tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-long-text.json`
  - asserts bounded input/root geometry, trailing-control non-overlap, IME cursor area inside the
    input, text-input horizontal overflow, offset range, and visible text inside the padded content
    viewport
  - suite redirect:
    `tools/diag-scripts/ui-gallery-button-group-input-group-long-text.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-button-group-input-group-long-text`)
  - current evidence:
    `target/fret-diag-button-group-input-group-long-text-text-visual-current/1778630553853/script.result.json`
  - current share pack:
    `target/fret-diag-button-group-input-group-long-text-text-visual-current/share/1778630553853.zip`
  - redirect-path evidence:
    `target/fret-diag-button-group-input-group-long-text-redirect-current/1778631486645/script.result.json`
  - redirect-path share pack:
    `target/fret-diag-button-group-input-group-long-text-redirect-current/share/1778631486645.zip`
  - visual proof:
    content width `625.2627`, viewport width `326.0`, offset/max offset `299.2627`, visible text
    bounds inside viewport `x=335.33334..661.33337`
- Combobox Input Group long-query visible-text gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-input-group-long-query-text.json`
  - asserts the searchable combobox input with an inline search addon keeps long-query text within
    its viewport, clamps horizontal offset, reports overflow, keeps the IME cursor inside bounds,
    and covers the measured text height
  - suite redirect:
    `tools/diag-scripts/ui-gallery-combobox-input-group-long-query-text.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-combobox-input-group-long-query-text`)
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-combobox/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - current evidence:
    `target/fret-diag-combobox-input-group-long-query-text-height-current/1778633183489/script.result.json`
  - current share pack:
    `target/fret-diag-combobox-input-group-long-query-text-height-current/share/1778633183489.zip`
  - visual proof after the TextInput viewport-height fix:
    viewport height `20.0`, clip height `20.0`, visible text height `20.0`, text run height `20.0`,
    content width `511.00684`, viewport width `170.0`, offset/max offset `341.00684`
- Command docs demo long-query visible-text gate:
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-docs-demo-long-query-text.json`
  - asserts the cmdk-style Command search input keeps long-query text inside its viewport, clamps
    horizontal offset, reports overflow, keeps the IME cursor inside bounds, and covers the measured
    text height
  - suite redirect:
    `tools/diag-scripts/ui-gallery-command-docs-demo-long-query-text.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-command-docs-demo-long-query-text`)
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-command/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - first-run harness failure evidence:
    `target/fret-diag/codex-command-long-query/sessions/1778633868562-103168/script.result.json`
  - current passing evidence:
    `target/fret-diag/codex-command-long-query-rerun/sessions/1778634588632-102660/script.result.json`
  - current share pack:
    `target/fret-diag/codex-command-long-query-rerun/sessions/1778634588632-102660/share/1778634605262.zip`
  - current screenshot:
    `target/fret-diag/codex-command-long-query-rerun/sessions/1778634588632-102660/screenshots/1778634607431-ui-gallery-command-docs-demo-long-query-text/window-4294967297-tick-80-frame-80.png`
- Input Basic + File long-text visible-text gate:
  `tools/diag-scripts/ui-gallery/input/ui-gallery-input-basic-and-file-long-text.json`
  - asserts a plain Input and the file-composition Input both expose direct editable text-field
    semantics, accept long values, report horizontal overflow, keep offset in range, keep visible
    text inside the viewport, keep the IME cursor inside bounds, and cover measured text height
  - suite redirect:
    `tools/diag-scripts/ui-gallery-input-basic-and-file-long-text.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-input-basic-and-file-long-text`)
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - first failed evidence, before the Basic Input builder-order fix:
    `target/fret-diag/codex-input-basic-file-long-text/sessions/1778635410854-100860/script.result.json`
  - second failed evidence, after assigning a unique id but before moving `.test_id(...)` before
    `.into_element(cx)`:
    `target/fret-diag/codex-input-basic-file-long-text-fixed/sessions/1778635816647-104372/script.result.json`
  - current passing evidence:
    `target/fret-diag/codex-input-basic-file-long-text-builder-fixed/sessions/1778636295426-16732/script.result.json`
  - current share pack:
    `target/fret-diag/codex-input-basic-file-long-text-builder-fixed/sessions/1778636295426-16732/share/1778636299109.zip`
  - current slices:
    `target/fret-diag/codex-input-basic-file-long-text-builder-fixed/sessions/1778636295426-16732/1778636303293-ui-gallery-input-basic-and-file-long-text/slice.ui-gallery-input-basic-control.json`,
    `target/fret-diag/codex-input-basic-file-long-text-builder-fixed/sessions/1778636295426-16732/1778636303293-ui-gallery-input-basic-and-file-long-text/slice.ui-gallery-input-file-control.json`
  - current screenshot:
    `target/fret-diag/codex-input-basic-file-long-text-builder-fixed/sessions/1778636295426-16732/screenshots/1778636303293-ui-gallery-input-basic-and-file-long-text/window-4294967297-tick-65-frame-65.png`
- Text-control authoring-surface gate:
  `apps/fret-ui-gallery/tests/ui_snippets_text_control_test_id_surface.rs`
  - reads UI Gallery diagnostics scripts under `tools/diag-scripts/ui-gallery`
  - collects `set_text_value` `test_id` targets and rejects snippet source chains that stamp the
    same id after `.into_element(cx)`
  - protects the F59 failure mode where a text-control id resolves to a landed wrapper instead of
    the editable text-field semantics node
  - command:
    `CARGO_TARGET_DIR=target/codex-diag cargo nextest run -p fret-ui-gallery --test ui_snippets_text_control_test_id_surface`
  - result: passed
  - Nextest run id: `89607014-877d-49c3-9f92-b3d3d12a68d7`
- Combobox responsive visible-bounds placement gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-open.json`
  - Asserts preferred/chosen bottom placement, `side_offset_px=6`, content left alignment with the
    trigger, and the documented responsive `150px` trigger / `200px` desktop popover width delta.
  - Popover first-open size-hint fix:
    `ecosystem/fret-ui-shadcn/src/popover.rs`
  - Focused tests:
    `popover_first_open_placement_size_prefers_explicit_hint` and
    `popover_stable_placement_size_respects_last_bounds_and_hints`
  - Evidence:
    `target/fret-diag/combobox-position-fixed/sessions/1778592265006-81796/1778592267296/script.result.json`
  - Trace verification:
    `desired.w=200`, `final_rect.w=200`, `chosen_side=bottom`, `side_offset_px=6`
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
- Menubar RTL tight-left collision gate:
  `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-rtl-submenu-tight-left-collision.json`
  - Asserts `ui-gallery-menubar-rtl-more` flips back to physical `right` when the preferred RTL
    inline-end side has insufficient left-room.
  - Suite membership:
    `tools/diag-scripts/suites/ui-gallery-rtl-smoke/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - Evidence:
    `target/fret-diag-menubar-rtl-tight-left/sessions/1778583596413-81824/script.result.json`
  - Layout sidecar:
    `target/fret-diag-menubar-rtl-tight-left/sessions/1778583596413-81824/1778583599421-ui-gallery-menubar-rtl-submenu-tight-left-collision.layout/layout.taffy.v1.json`
  - Screenshot:
    `target/fret-diag-menubar-rtl-tight-left/sessions/1778583596413-81824/screenshots/1778583599456-ui-gallery-menubar-rtl-submenu-tight-left-collision/window-4294967297-tick-40-frame-40.png`
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
