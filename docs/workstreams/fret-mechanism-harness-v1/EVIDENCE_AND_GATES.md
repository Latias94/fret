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

## Layout Primitives Expansion Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_layout_primitives_match_oracles -- --nocapture
```

Current layout primitive fixture coverage includes text measurement/paint agreement metrics for
column wrap width, max-width row wrap width, and overflow/scale constraints.

## Suite Lint Policy Gates

```powershell
cargo test --profile dev-fast -p fret-diag --lib suite_lint_policy -- --nocapture
cargo test --profile dev-fast -p fret-diag --lib lint_warning_budget -- --nocapture
cargo test --profile dev-fast -p fret-diag --lib maybe_run_suite_script_lint -- --nocapture
cargo test --profile dev-fast -p fret-diag --lib finalize_suite_script_success_tail_records_row_when_lint_and_post_run_skip -- --nocapture
python tools/check_diag_scripts_registry.py
cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev
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

## Anchored Cross-Root Coordinate Gates

```powershell
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/anchored_cross_root_coordinate_v1.json
cargo test -p fret-ui --lib mechanism_harness_anchored_cross_root_coordinate_matches_oracles -- --nocapture
cargo nextest run -p fret-ui-kit outer_bounds_with_window_margin --no-fail-fast
$env:CARGO_BUILD_JOBS='1'; cargo test -p fret-ui-shadcn --lib popover_first_open_placement_size_prefers_explicit_hint -- --nocapture
$env:CARGO_BUILD_JOBS='1'; cargo test -p fret-ui-shadcn --lib hover_card_anchor_override_uses_anchor_bounds_for_placement -- --nocapture
$env:CARGO_BUILD_JOBS='1'; cargo test -p fret-ui-shadcn --lib tooltip_anchor_override_uses_anchor_bounds_for_placement -- --nocapture
$env:CARGO_BUILD_JOBS='1'; cargo test -p fret-ui-shadcn --lib dropdown_menu_portal_escapes_overflow_clip_ancestor -- --nocapture
$env:CARGO_BUILD_JOBS='1'; cargo test -p fret-ui-shadcn --lib context_menu_submenu_keyboard_open_transfers_focus_and_arrow_left_restores_focus -- --nocapture
$env:CARGO_BUILD_JOBS='1'; cargo test -p fret-ui-shadcn --lib menubar_submenu_opens_on_arrow_right_and_closes_on_arrow_left_restoring_focus -- --nocapture
```

## View-Cache and Root-Boundary Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_layout_dirty_invalidation_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_view_cache_lifecycle_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib view_cache -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib scroll_contained_view_cache_dirty_does_not_force_direct_child_root_invalidation -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib layout_request_build_roots_classify_view_cache_layout_dirty_expansion -- --nocapture
```

Current synthetic evidence anchors:

- View-cache lifecycle fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/view_cache_lifecycle_v1.json`
  - runner:
    `crates/fret-ui/src/declarative/tests/view_cache_lifecycle_harness.rs`
  - proof:
    asserts clean cache-hit reuse, retained element state, cache-key misses, RAF invalidation,
    model-observation preservation across cache-hit frames, unrelated model scoping,
    inspection-mode bypass, and layout-query next-frame invalidation.
  - current command:
    `cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_view_cache_lifecycle_matches_oracles -- --nocapture`
  - current result:
    passed.

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
$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json --dir target/fret-diag-datatable-filter-shrink-inputs-change-after-layout-prev-input-fix --session-auto --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
$env:FRET_UI_GALLERY_VIEW_CACHE='1'
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json --dir target/fret-diag-datatable-view-cache-filter-shrink-inputs-change --session-auto --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-scroll-to-rtl-field.json --dir target/fret-diag-rtl-scroll-idle-stability-v2 --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
cargo nextest run -p fret-diag-protocol script_v2_roundtrip_ui_gallery_scroll_area_expand_at_bottom --no-fail-fast
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-expand-at-bottom.json --dir target/fret-diag-scroll-area-expand-at-bottom-v1 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target/debug/fret-ui-gallery.exe
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-rtl-idle-stability.json --dir target/fret-diag-scroll-area-rtl-idle-stability --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/table/ui-gallery-table-rtl-idle-stability.json --dir target/fret-diag-table-rtl-idle-stability --session-auto --pack --ai-packet --include-screenshots --launch -- target/release/fret-ui-gallery.exe
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-rtl-idle-stability.json --dir target/fret-diag-data-table-rtl-idle-stability --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
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
- DataTable retained filter-shrink input-change runtime gate:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-data-table-retained/suite.json`
  - proof:
    drives the real DataTable torture page with `FRET_UI_GALLERY_DATA_TABLE_RETAINED=1`, scrolls the
    retained table, applies the global filter `Process 123`, observes `GlobalFilter: Process 123`,
    and asserts at least one layout-sourced virtual-list window record with
    `reason=inputs_change` and `apply_mode=retained_reconcile`.
  - evidence:
    `target/fret-diag-datatable-filter-shrink-inputs-change-after-layout-prev-input-fix/sessions/1778743224380-134968/1778743227800`
  - share pack:
    `target/fret-diag-datatable-filter-shrink-inputs-change-after-layout-prev-input-fix/sessions/1778743224380-134968/share/1778743227800.zip`
- DataTable view-cache filter-shrink input-change runtime gate:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-data-table-view-cache-torture/suite.json`
  - proof:
    drives the real DataTable torture page with `FRET_UI_GALLERY_VIEW_CACHE=1`, scrolls the table,
    applies the global filter `Process 123`, observes `GlobalFilter: Process 123`, and asserts at
    least one layout-sourced virtual-list window record with `reason=inputs_change`,
    `apply_mode=non_retained_rerender`, and
    `invalidation_detail=scroll_handle_inputs_change_window_update`.
  - first failed precondition evidence:
    `target/fret-diag-datatable-filter-shrink-nonretained-inputs-change/sessions/1778744865979-149372/1778744869344/ai.packet`
  - current evidence:
    `target/fret-diag-datatable-view-cache-filter-shrink-inputs-change/sessions/1778745510540-145220/1778745514577`
  - share pack:
    `target/fret-diag-datatable-view-cache-filter-shrink-inputs-change/sessions/1778745510540-145220/share/1778745514577.zip`
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
- ScrollArea content-growth extent gate:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-expand-at-bottom.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-scroll-area/suite.json`
  - proof:
    starts with `y_max=0`, clicks the dynamic content-growth toggle, waits for `y_max` to become
    non-zero and stable, then wheels the grown viewport and asserts `y != 0`.
  - evidence:
    `target/fret-diag-scroll-area-expand-at-bottom-v1/sessions/1778953288892-97788/1778953292783`
  - share pack:
    `target/fret-diag-scroll-area-expand-at-bottom-v1/sessions/1778953288892-97788/share/1778953292783.zip`
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
- DataTable RTL post-scroll idle-stability gate:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-rtl-idle-stability.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-rtl-smoke/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - proof:
    scrolls the DataTable page to `ui-gallery-data-table-rtl-root`, waits for the root and footer
    bounds to settle, then samples `ui-gallery-content-viewport` for 60 no-input frames.
  - evidence:
    `target/fret-diag-data-table-rtl-idle-stability/sessions/1778746329247-149040/1778746333478`
  - share pack:
    `target/fret-diag-data-table-rtl-idle-stability/sessions/1778746329247-149040/share/1778746333478.zip`

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
cargo nextest run -p fret-diag-protocol predicate_captured_is_serializes --no-fail-fast
cargo nextest run -p fret-diag-protocol pointer_session_step_pointer_id_defaults_and_round_trips --no-fail-fast
cargo nextest run -p fret-mechanism-harness captured_is_oracle_tracks_current_capture_owner --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics ui_diagnostics::predicate_tests::captured_is_matches_semantics_capture_owner --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics no_frame_pointer_move --no-fail-fast
cargo nextest run -p fret-ui semantics_snapshot_dirty_gate_tracks_pointer_capture_owner --no-fail-fast
cargo nextest run -p fret-diag-protocol script_v2_roundtrip_dock_viewport_capture_active_is_predicate --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics dock_viewport_capture_active_matches_docking_snapshot --no-fail-fast
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json --dir target/fret-diag-scrollbar-drag-pointer-capture --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-pointer-cancel-release.json --dir target/fret-diag-scrollbar-drag-cancel-release-v2 --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-multipointer-underlay-touch.json --dir target/fret-diag-scrollbar-drag-multipointer-underlay-touch-v5 --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard-dev.exe diag run tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-dock-drag-suppresses-viewport-touch.json --dir target/fret-diag-docking-multiwindow-dock-drag-suppresses-viewport-touch-v1 --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/docking_arbitration_demo.exe
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-branch-corridor-routing.json --dir target/fret-diag-context-menu-submenu-branch-corridor-routing-v1 --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-context-menu-occlusion-wheel-pass-through.json --dir target/fret-diag-context-menu-occlusion-wheel-structured-v2 --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
```

Current runtime evidence:

- Scrollbar drag pointer-capture lifecycle and owner:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json`
  - asserts `input_pointer_capture_active_is active=true` after `pointer_down` on
    `ui-gallery-scroll-area-drag-baseline-y-scrollbar`, asserts it stays true during the drag,
    asserts `captured_is=true` for the scrollbar owner, keeps the existing
    `semantics_scroll_approx_eq y=20` scroll-progress oracle, and asserts both `active=false` and
    `captured_is=false` after `pointer_up`.
  - current owner run evidence:
    `target/fret-diag-scrollbar-drag-owner-content-growth-v3/sessions/1778758997885-141672/1778759002275`
  - current owner share pack:
    `target/fret-diag-scrollbar-drag-owner-content-growth-v3/sessions/1778758997885-141672/share/1778759002275.zip`
  - result:
    passed after the mechanism fix. The first owner-level runtime gate exposed stale semantics
    capture-owner publication; `UiTree::request_semantics_snapshot_if_dirty()` now refreshes when
    focus/capture owner input state differs from the current snapshot.
- Scrollbar drag pointer-cancel release:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-pointer-cancel-release.json`
  - asserts `active=true` and `captured_is=true` after `pointer_down`, dispatches
    `pointer_cancel`, waits two frames, and asserts `active=false` plus `captured_is=false`.
  - current run evidence:
    `target/fret-diag-scrollbar-drag-cancel-release-v2/sessions/1778758997885-128560/1778759002526`
  - current share pack:
    `target/fret-diag-scrollbar-drag-cancel-release-v2/sessions/1778758997885-128560/share/1778759002526.zip`
  - result:
    passed; no recipe defect reproduced. This locks release-on-cancel behavior through the same
    owner-level semantics predicate that found the stale snapshot mechanism defect.
- Scrollbar drag multi-pointer underlay touch:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-multipointer-underlay-touch.json`
  - asserts pointer `0` captures the scrollbar, pointer `1` performs a touch down/up on the
    viewport probe while capture is still held, the scrollbar remains the capture owner, and
    pointer `0` cancel releases capture at the end.
  - current run evidence:
    `target/fret-diag-scrollbar-drag-multipointer-underlay-touch-v5/sessions/1778766146942-159636/1778766151512`
  - current share pack:
    `target/fret-diag-scrollbar-drag-multipointer-underlay-touch-v5/sessions/1778766146942-159636/share/1778766151512.zip`
  - companion baseline evidence after adding the viewport probe:
    `target/fret-diag-scrollbar-drag-baseline-content-growth-after-probe-v1/sessions/1778765762647-151848/1778765769291`
  - result:
    passed; no core pointer-capture routing defect reproduced. The fixed defect was a runtime
    diagnostics harness gap: schema-v2 pointer-session steps and the runner could only model one
    hardcoded `PointerId(0)` session, so they could not probe captured-underlay behavior with a
    second pointer.
- Cross-window docking drag suppresses secondary viewport capture:
  `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-dock-drag-suppresses-viewport-touch.json`
  - tears off a dock tab into a second OS window, starts a dock drag from the overlapping moving
    window with pointer `0`, probes the main-window viewport with pointer `1` touch down/up, and
    asserts `dock_drag_active_is=true` while `dock_viewport_capture_active_is=false`.
  - current run evidence:
    `target/fret-diag-docking-multiwindow-dock-drag-suppresses-viewport-touch-v1/sessions/1778768809522-14036/1778768814411`
  - current share pack:
    `target/fret-diag-docking-multiwindow-dock-drag-suppresses-viewport-touch-v1/sessions/1778768809522-14036/share/1778768814411.zip`
  - result:
    passed; no core docking defect reproduced. The fixed defect was a diagnostics harness
    observability gap: runtime scripts could not directly assert absence of a competing docking
    viewport capture.
- ContextMenu submenu branch/corridor routing:
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-branch-corridor-routing.json`
  - proves entering submenu content and returning to the parent trigger keeps the nested submenu
    open, moving to another parent-menu item closes the nested submenu while preserving the root
    menu, moving away from the trigger closes the nested submenu, and sweeping toward the submenu
    keeps it open.
  - current run evidence:
    `target/fret-diag-context-menu-submenu-branch-corridor-routing-v1/sessions/1778770209711-161288/1778770215056`
  - current share pack:
    `target/fret-diag-context-menu-submenu-branch-corridor-routing-v1/sessions/1778770209711-161288/share/1778770215056.zip`
  - result:
    passed; no core hit-test or ContextMenu recipe defect reproduced. The fixed defect was runtime
    coverage weakness: branch/corridor behavior was only indirectly covered by placement traces.
  - current selector-fix rerun:
    `target/fret-diag-context-menu-submenu-overlay-content-id-fix-v1/sessions/1778820109246-70216`
  - selector proof:
    the page/snippet container id `ui-gallery-context-menu-submenu-content` and the mounted overlay
    panel id `ui-gallery-context-menu-submenu-overlay-content` each resolve to exactly one node in
    `1778820148136-ui-gallery-context-menu-submenu-branch-corridor-routing.layout/bundle.schema2.json`.
  - overlay/focus suite proof after the selector fix:
    `target/fret-diag-overlay-focus-suite-after-context-menu-selector-fix-v1/sessions/1778820202372-67920/suite.summary.json`
    has `status=passed`, `stage_counts.passed=8`, and zero ContextMenu branch/corridor lint
    findings.
- Context-menu pointer occlusion wheel pass-through:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-context-menu-occlusion-wheel-pass-through.json`
  - asserts `ui-gallery-content-viewport.scroll.y_max != 0` and `scroll.y == 0` before the wheel,
    wheels `ui-gallery-overlay-reset` under `BlockMouseExceptScroll`, then asserts
    `ui-gallery-content-viewport.scroll.y != 0` while `ui-gallery-context-content` still exists.
  - current run result:
    `target/fret-diag-context-menu-occlusion-wheel-structured-v2/sessions/1778749334444-150476/script.result.json`
  - current bundle:
    `target/fret-diag-context-menu-occlusion-wheel-structured-v2/sessions/1778749334444-150476/1778749343795-ui-gallery-context-menu-occlusion-wheel-pass-through/bundle.schema2.json`
  - current share pack:
    `target/fret-diag-context-menu-occlusion-wheel-structured-v2/sessions/1778749334444-150476/share/1778749337977.zip`
  - current screenshot after wheel:
    `target/fret-diag-context-menu-occlusion-wheel-structured-v2/sessions/1778749334444-150476/screenshots/1778749343662-ui-gallery-context-menu-occlusion-wheel-pass-through-after/window-4294967297-tick-49-frame-49.png`
  - trace proof from the final bundle: `pointer_occlusion=block_mouse_except_scroll`,
    `pointer_capture_active=false`, and `ui-gallery-content-viewport.scroll.y=535.3333740234375`.

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
cargo nextest run -p fret-mechanism-harness semantics_value_state_actions_and_structured_metadata_are_queryable --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics default_predicates_exclude_semantics_hidden_subtrees_but_flags_remain_observable --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics expanded_is_matches_semantics_expanded_flag --no-fail-fast
cargo nextest run -p fret-diag-protocol predicate_raw_semantics_hidden_is_serializes_and_deserializes script_v2_roundtrip_ui_gallery_separator_decorative_hidden_semantics --no-fail-fast
cargo nextest run -p fret-diag-protocol predicate_expanded_is_serializes_and_deserializes script_v2_roundtrip_ui_gallery_accordion_usage_toggle --no-fail-fast
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_semantics_relations_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_combobox_active_descendant_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib text_input_semantics_controls_element_is_exposed -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib text_input_semantics_active_descendant_element_is_exposed -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib declarative_attach_semantics_can_override_state_and_relations -- --nocapture
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/separator/ui-gallery-separator-decorative-hidden-semantics.json --dir target/fret-diag-separator-decorative-hidden-semantics-v3 --launch -- cargo run -p fret-ui-gallery
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-usage-toggle.json --dir target/fret-diag-accordion-expanded-semantics-v1 --launch -- cargo run -p fret-ui-gallery
cargo nextest run -p fret-diag-protocol predicate_selected_is_serializes_and_deserializes script_v2_roundtrip_ui_gallery_select_commit_and_label_update --no-fail-fast
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/select/ui-gallery-select-commit-and-label-update.json --dir target/fret-diag-select-selected-state-mutation-v2 --launch -- cargo run -p fret-ui-gallery
cargo nextest run -p fret-diag-protocol script_v2_roundtrip_ui_gallery_tabs_selected_state_mutation --no-fail-fast
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-selected-state-mutation.json --dir target/fret-diag-tabs-selected-state-mutation-v1 --launch -- cargo run -p fret-ui-gallery
```

Current semantics runtime evidence anchors:

- Separator decorative hidden semantics gate:
  `tools/diag-scripts/ui-gallery/separator/ui-gallery-separator-decorative-hidden-semantics.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - protocol predicate:
    `crates/fret-diag-protocol/src/lib.rs` (`raw_semantics_hidden_is`)
  - runtime predicate and selector implementation:
    `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/selector.rs`
  - current evidence:
    `target/fret-diag-separator-decorative-hidden-semantics-v3/script.result.json`
  - current bundle artifact dir:
    `target/fret-diag-separator-decorative-hidden-semantics-v3/1778775274012-ui-gallery-separator-decorative-hidden-semantics`
  - result:
    passed; the fixed defect was a diagnostics/runtime selector mismatch where hidden semantics
    could satisfy default selectors or become hard to assert through a reusable raw predicate.
- Accordion expanded-state mutation gate:
  `tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-usage-toggle.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - protocol predicate:
    `crates/fret-diag-protocol/src/lib.rs` (`expanded_is`)
  - runtime predicate:
    `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`
  - current evidence:
    `target/fret-diag-accordion-expanded-semantics-v1/script.result.json`
  - current bundle artifact dirs:
    `target/fret-diag-accordion-expanded-semantics-v1/1778777596692-ui-gallery-accordion-usage-toggle-closed`,
    `target/fret-diag-accordion-expanded-semantics-v1/1778777597965-ui-gallery-accordion-usage-toggle-open`
  - result:
    passed; no Accordion recipe defect reproduced. The fixed defect was a harness observability gap:
    runtime scripts could not assert expanded-state mutation directly.

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
target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-typeahead-commit-banana.json --dir target/fret-diag-combobox-typeahead-commit-banana-v2 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe
```

Current runtime evidence:

- Combobox typeahead commit runtime gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-typeahead-commit-banana.json`
  - runtime assertions:
    navigates to Combobox, scrolls the demo trigger into the content viewport, uses stable clicks to
    open/reopen the popover, filters with `ban`, asserts Banana becomes the active item, commits
    with Enter, verifies the selected label updates to `Selected: banana`, reopens, and asserts
    Banana has `selected_is=true`.
  - focused failure evidence before the precondition fix:
    `target/fret-diag-combobox-suite-responsive-resize-v1/sessions/1778847873963-70928/script.result.json`
  - failure proof:
    step 14 hit-test trace recorded `clamped_outside_window=true`, intended trigger
    `y_px=3831.333`, click `y_px=720`, and `blocking_reason=no_hit`.
  - runtime evidence after the fix:
    `target/fret-diag-combobox-typeahead-commit-banana-v2/sessions/1778848767000-28640/1778848774825/script.result.json`
  - AI packet:
    `target/fret-diag-combobox-typeahead-commit-banana-v2/sessions/1778848767000-28640/1778848774825/ai.packet`
  - packed evidence:
    `target/fret-diag-combobox-typeahead-commit-banana-v2/sessions/1778848767000-28640/share/1778848774825.zip`
  - component-suite proof:
    `target/fret-diag-combobox-suite-responsive-resize-v2/sessions/1778848841597-52396/suite.summary.json`
  - suite result:
    passed, 23 scripts; `reason_code_counts={}`; typeahead row passed with run id `1778849398883`.
  - roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_typeahead_commit_banana --no-fail-fast`
  - roundtrip result:
    passed, 1 test; Nextest run id `87ecad1d-1d24-424a-a0ac-571d46d0963d`.

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
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-geometry.json --dir target/fret-diag-combobox-rtl-long-text-audit --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
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
- Combobox RTL long-text trigger/option geometry gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-geometry.json`
  - asserts trigger label width budget, physical-left RTL chevron inset, label-after-chevron
    spacing, chrome-relative vertical centering, content-shell top collision flip with
    `side_offset_px=6`, option label width budget, physical-right RTL checkmark inset, and
    label-before-checkmark spacing.
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-combobox/suite.json`,
    `tools/diag-scripts/suites/ui-gallery-rtl-smoke/suite.json`, and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - current audit evidence:
    `target/fret-diag-combobox-rtl-long-text-audit/sessions/1778747708234-13552/1778747711700/script.result.json`
  - current audit share pack:
    `target/fret-diag-combobox-rtl-long-text-audit/sessions/1778747708234-13552/share/1778747711700.zip`
  - current audit layout sidecar:
    `target/fret-diag-combobox-rtl-long-text-audit/sessions/1778747708234-13552/1778747713455-ui-gallery-combobox-rtl-long-text-open.layout/layout.taffy.v1.json`
  - current audit screenshot:
    `target/fret-diag-combobox-rtl-long-text-audit/sessions/1778747708234-13552/screenshots/1778747713519-ui-gallery-combobox-rtl-long-text-open/window-4294967297-tick-34-frame-34.png`
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
target/dev-fast/fretboard-dev.exe diag suite ui-gallery-button-group --dir target/fret-diag-button-group-family-suite-v3 --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe
cargo nextest run --cargo-profile dev-fast -p fret-diag lint_treats_labelled_by_relation_as_accessible_name_source --no-fail-fast
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

- Button Group family suite:
  `tools/diag-scripts/suites/ui-gallery-button-group/suite.json`
  - runs docs, demo, icon, size, ButtonGroupText, Input Group, long-text, RTL addon, input fill,
    separator, accessibility, and Select screenshot/geometry/lint coverage through one durable
    family entry point.
  - current result:
    `target/fret-diag-button-group-family-suite-v3/sessions/1778810828217-59768/suite.summary.json`
  - outcome: `status=passed`, `stage_counts.passed=13`, `reason_code_counts={}`, and all generated
    lint reports have `warning_issues=0`.
- Diagnostics lint labelled-by accessible-name gate:
  `crates/fret-diag/src/lint.rs`
  - test: `lint_treats_labelled_by_relation_as_accessible_name_source`
  - run id: `d3b80633-a713-4d44-8b88-5fb8af5405a6`.
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
- `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-branch-corridor-routing.json`
- `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json`
- `tools/diag-scripts/suites/fret-mechanism-harness-overlay-focus/suite.json`

Current evidence:

- `target/fret-diag-overlay-focus-suite-after-context-menu-selector-fix-v1/sessions/1778820202372-67920/suite.summary.json`
  - `status=passed`
  - `stage_counts.passed=8`
  - `reason_code_counts={}`
  - generated lint reports have zero errors and zero warnings for the promoted ContextMenu and
    DropdownMenu additions.

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
- Pointer occlusion runtime gate:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-context-menu-occlusion-wheel-pass-through.json`
- Pointer-capture lifecycle predicate:
  `crates/fret-diag-protocol/src/lib.rs` (`input_pointer_capture_active_is`)
- Pointer-capture lifecycle runtime gate:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json`
- Pointer-capture owner predicate:
  `crates/fret-diag-protocol/src/lib.rs` (`captured_is`)
- Pointer-capture owner runtime predicate evaluator:
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`
- Pointer-capture owner synthetic oracle:
  `crates/fret-mechanism-harness/src/oracle.rs`
- Pointer-capture semantics dirty-gate fix:
  `crates/fret-ui/src/tree/ui_tree_semantics.rs`
- Pointer-capture owner and cancel runtime gates:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json`,
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-pointer-cancel-release.json`
- Multi-pointer pointer-session protocol support:
  `crates/fret-diag-protocol/src/lib.rs` (`pointer_id` on pointer-session steps),
  `crates/fret-diag-protocol/src/builder.rs`
- Multi-pointer runtime session support:
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer_session.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- ScrollArea multi-pointer diagnostics surface:
  `apps/fret-ui-gallery/src/ui/diagnostics/scroll_area/drag_baseline.rs`
- Multi-pointer captured-underlay runtime gate:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-multipointer-underlay-touch.json`
- ScrollArea diagnostics suite and registry:
  `tools/diag-scripts/suites/ui-gallery-scroll-area/suite.json`,
  `tools/diag-scripts/index.json`
- Dock viewport-capture predicate:
  `crates/fret-diag-protocol/src/lib.rs` (`dock_viewport_capture_active_is`),
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`
- Cross-window docking multi-pointer runtime gate:
  `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-dock-drag-suppresses-viewport-touch.json`
- ContextMenu branch/corridor runtime gate:
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-branch-corridor-routing.json`
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
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-escape-focus-restore.json`,
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-branch-corridor-routing.json`,
  `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json`
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
- Composite active-descendant runtime gates:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-auto-highlight-disabled-none-on-open.json`,
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-auto-highlight-first-match.json`,
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-palette-controlled-selection-arrowdown.json`,
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-palette-controlled-selection-value.json`
  - assert that active item semantics remain empty when Combobox auto-highlight is disabled, move
    to the first matching Combobox item when auto-highlight is enabled, follow Command controlled
    selection value changes, and advance after Command ArrowDown while focus remains on the
    composite input
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-combobox/suite.json`,
    `tools/diag-scripts/suites/ui-gallery-command/suite.json`, and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - diagnostics catalog entries:
    `tools/diag-scripts/index.json`
    (`ui-gallery-combobox-auto-highlight-disabled-none-on-open`,
    `ui-gallery-combobox-auto-highlight-first-match`,
    `ui-gallery-command-palette-controlled-selection-arrowdown`,
    `ui-gallery-command-palette-controlled-selection-value`)
  - current evidence:
    `target/fret-diag-combobox-active-descendant-disabled-v1/script.result.json`,
    `target/fret-diag-combobox-active-descendant-first-match-v1/script.result.json`,
    `target/fret-diag-command-active-descendant-arrowdown-v1/script.result.json`,
    `target/fret-diag-command-active-descendant-value-v1/script.result.json`
  - protocol gate:
    `cargo nextest run -p fret-diag-protocol --test script_json_roundtrip --no-fail-fast`
  - result: passed, Nextest run id `556c42bf-ff61-4bcb-aa90-5a12a27ba7c9`
- Sonner live-region mutation gate:
  `tools/diag-scripts/ui-gallery/sonner/ui-gallery-sonner-live-region-mutation.json`
  - asserts the `Notifications` toast viewport is absent before showing a toast, exposes
    `semantics_live_is=polite` and `semantics_live_atomic_is=false` while the toast is mounted, and
    disappears after swipe dismissal
  - suite redirect:
    `tools/diag-scripts/ui-gallery-sonner-live-region-mutation.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-sonner-docs/suite.json`,
    `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`, and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-sonner-live-region-mutation`)
  - current evidence:
    `target/fret-diag-sonner-live-region-mutation-v2/script.result.json`
  - current bundle artifact dirs:
    `target/fret-diag-sonner-live-region-mutation-v2/1778783117449-ui-gallery-sonner-live-region-open`,
    `target/fret-diag-sonner-live-region-mutation-v2/1778783117667-ui-gallery-sonner-live-region-closed`
  - focused gates:
    `cargo nextest run -p fret-diag-protocol predicate_semantics_live_is_serializes_and_deserializes predicate_semantics_live_atomic_is_serializes_and_deserializes script_v2_roundtrip_ui_gallery_sonner_live_region_mutation --no-fail-fast`,
    `cargo nextest run -p fret-mechanism-harness semantics_value_state_actions_and_structured_metadata_are_queryable --no-fail-fast`,
    `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics semantics_live_predicates_match_semantics_live_flags --no-fail-fast`
- Select selected-state mutation gate:
  `tools/diag-scripts/ui-gallery/select/ui-gallery-select-commit-and-label-update.json`
  - asserts Select commits Banana, updates the external selected-label text, restores focus to the
    trigger, reopens the popup, and exposes Banana as `selected_is=true` while Apple is
    `selected_is=false`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-select-commit-and-label-update`)
  - first failed evidence before overlay-placement/bounds stabilization:
    `target/fret-diag-select-selected-state-mutation-v1/script.result.json`
  - failure bundle:
    `target/fret-diag-select-selected-state-mutation-v1/1778784037422-script-step-0019-wait_until-timeout`
  - current passing evidence:
    `target/fret-diag-select-selected-state-mutation-v2/script.result.json`
  - protocol/script gate:
    `cargo nextest run -p fret-diag-protocol predicate_selected_is_serializes_and_deserializes script_v2_roundtrip_ui_gallery_select_commit_and_label_update --no-fail-fast`
  - result:
    passed after the script stability fix. No Select recipe defect was reproduced; the fixed defect
    was a diagnostics harness timing gap where item clicks could run before overlay placement and
    visible bounds were ready.
- Tabs selected-state mutation gate:
  `tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-selected-state-mutation.json`
  - asserts the Tabs demo starts with Account `selected_is=true` and Password
    `selected_is=false`, clicks the Password trigger, then asserts Account becomes false and
    Password becomes true while focus moves to Password
  - suite redirect:
    `tools/diag-scripts/ui-gallery-tabs-selected-state-mutation.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-tabs-selected-state-mutation`)
  - current evidence:
    `target/fret-diag-tabs-selected-state-mutation-v1/script.result.json`
  - current bundle artifact dir:
    `target/fret-diag-tabs-selected-state-mutation-v1/1778785691560-ui-gallery-tabs-selected-state-mutation`
  - protocol/script gate:
    `cargo nextest run -p fret-diag-protocol script_v2_roundtrip_ui_gallery_tabs_selected_state_mutation --no-fail-fast`
  - result:
    passed; no Tabs mechanism or recipe defect was reproduced. This gate broadens selected-state
    coverage beyond Select's overlay-backed item semantics to inline tab triggers.
- Command collection metadata mutation gate:
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-scrollable-collection-metadata-mutation.json`
  - asserts the shadcn Command scrollable dialog exposes `Code Editor` as item 23/23 before
    filtering, then updates it to item 1/1 after filtering to `code editor`
  - suite redirect:
    `tools/diag-scripts/ui-gallery-command-scrollable-collection-metadata-mutation.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-command/suite.json`,
    `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`, and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-command-scrollable-collection-metadata-mutation`)
  - current evidence:
    `target/fret-diag-command-collection-metadata-mutation-v1/script.result.json`
  - current bundle artifact dir:
    `target/fret-diag-command-collection-metadata-mutation-v1/1778787013289-ui-gallery-command-scrollable-collection-metadata-mutation`
  - focused gates:
    `cargo nextest run -p fret-diag-protocol predicate_collection_position_serializes_and_deserializes script_v2_roundtrip_ui_gallery_command_scrollable_collection_metadata_mutation --no-fail-fast`,
    `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics collection_metadata_predicates_match_semantics_position_fields --no-fail-fast`
  - result:
    passed; no Command mechanism or recipe defect was reproduced. This closes the first shadcn
    runtime mutation gate for collection metadata and leaves pagination/windowed reuse as the next
    higher-risk collection semantics gap.
- DataTable pagination collection metadata gate:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-default-pagination-collection-metadata.json`
  - asserts the default DataTable row anchors expose page-local collection metadata across page
    changes: page 1 rows 1/2 and 2/2, page 2 rows 1/2 and 2/2, and final page row 1/1
  - suite redirect:
    `tools/diag-scripts/ui-gallery-data-table-default-pagination-collection-metadata.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-data-table/suite.json`,
    `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`, and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json`
    (`ui-gallery-data-table-default-pagination-collection-metadata`)
  - first failed evidence before the `Next` bounds gate:
    `target/fret-diag-data-table-pagination-collection-metadata-v1/script.result.json`
  - failure bundle:
    `target/fret-diag-data-table-pagination-collection-metadata-v1/1778787896483-script-step-0018-wait_until-timeout`
  - current passing evidence:
    `target/fret-diag-data-table-pagination-collection-metadata-v2/script.result.json`
  - current bundle artifact dir:
    `target/fret-diag-data-table-pagination-collection-metadata-v2/1778788248207-ui-gallery-data-table-default-pagination-collection-metadata`
  - focused gate:
    `cargo nextest run -p fret-diag-protocol script_v2_roundtrip_ui_gallery_data_table_default_pagination_collection_metadata --no-fail-fast`
  - result:
    passed after the script scrolled `Next` into view and asserted visible bounds before clicking.
    No DataTable pagination or core semantics defect was reproduced; retained/windowed row reuse
    remains the next collection-metadata risk.
- Retained Virtual List collection metadata bounce gate:
  `tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-retained-collection-metadata-bounce.json`
  - asserts row-root `ListItem` collection metadata on Virtual List Torture before scrolling
    (row 0 is 1/10000), after a retained boundary scroll (row 25 is 26/10000 and row 0 is
    detached), and after bouncing back (row 0 returns to 1/10000)
  - mechanism support:
    `crates/fret-ui/src/element.rs` and
    `crates/fret-ui/src/declarative/host_widget/semantics.rs`
  - UI Gallery anchors:
    `apps/fret-ui-gallery/src/ui/previews/pages/harness/virtual_list_torture.rs`
  - suite redirect:
    `tools/diag-scripts/ui-gallery-virtual-list-retained-collection-metadata-bounce.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-vlist-window-boundary-retained/suite.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json`
    (`ui-gallery-virtual-list-retained-collection-metadata-bounce`)
  - focused mechanism gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui declarative_attach_semantics_can_stamp_collection_metadata --no-fail-fast`
  - focused script gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_virtual_list_retained_collection_metadata_bounce --no-fail-fast`
  - current passing evidence:
    `target/fret-diag-vlist-retained-collection-metadata-bounce-v3/sessions/1778790236440-163244/1778790240041/script.result.json`
  - current AI packet:
    `target/fret-diag-vlist-retained-collection-metadata-bounce-v3/sessions/1778790236440-163244/1778790240041/ai.packet`
  - current retained suite summary:
    `target/fret-diag-vlist-window-boundary-retained-after-collection-metadata-v2/sessions/1778790349931-163108/suite.summary.json`
  - ecosystem compile guard:
    `cargo check --profile dev-fast -p fret-ui-material3 --all-targets`
  - result:
    passed after narrowing the new script to collection metadata plus retained attach/detach. The
    slice found a mechanism observability gap: generic semantics surfaces could not previously
    stamp collection metadata without pressable policy.
- Retained Tree hierarchy semantics mutation gate:
  `tools/diag-scripts/ui-gallery/tree/ui-gallery-tree-retained-hierarchy-semantics-toggle.json`
  - asserts root/folder/leaf `level_is`, parent-row `expanded_is`, child detachment after collapse,
    and restored hierarchy metadata after expansion under retained row reuse
  - suite redirect:
    `tools/diag-scripts/ui-gallery-tree-retained-hierarchy-semantics-toggle.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-tree-retained/suite.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-tree-retained-hierarchy-semantics-toggle`)
  - mechanism support:
    `crates/fret-diag-protocol/src/lib.rs`, `crates/fret-diag-protocol/src/builder.rs`,
    `crates/fret-mechanism-harness/src/observe.rs`, `crates/fret-mechanism-harness/src/oracle.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`, and
    `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`
  - component support:
    `ecosystem/fret-ui-kit/src/declarative/tree.rs` and
    `ecosystem/fret-ui-kit/src/declarative/file_tree.rs`
  - focused gates:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_level_is_serializes_and_deserializes script_v2_roundtrip_ui_gallery_tree_retained_hierarchy_semantics_toggle --no-fail-fast`,
    `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics level_is_matches_semantics_hierarchy_level --no-fail-fast`,
    `cargo nextest run --cargo-profile dev-fast -p fret-mechanism-harness semantics_value_state_actions_and_structured_metadata_are_queryable --no-fail-fast`,
    `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_semantics_relations_match_oracles --no-fail-fast`, and
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit tree_item_a11y select_tree_item toggle_tree_item file_tree_item_a11y --no-fail-fast`
  - first valid failed evidence:
    `target/fret-diag-tree-retained-hierarchy-semantics-v3/sessions/1778793173277-91896/1778793181464/script.result.json`
  - failure proof:
    `tree.toggle.0` was dispatched from `ui-gallery-tree-row-0-toggle` with `handled=false`, so
    `ui-gallery-tree-row-0` remained `expanded=true` after collapse.
  - current passing evidence:
    `target/fret-diag-tree-retained-hierarchy-semantics-v5/sessions/1778793872096-160900/1778793880135/script.result.json`
  - current AI packet:
    `target/fret-diag-tree-retained-hierarchy-semantics-v5/sessions/1778793872096-160900/1778793880135/ai.packet`
  - current share pack:
    `target/fret-diag-tree-retained-hierarchy-semantics-v5/sessions/1778793872096-160900/share/1778793880135.zip`
  - current retained Tree suite summary:
    `target/fret-diag-tree-retained-suite-after-hierarchy-semantics-v2/sessions/1778793907189-162608/suite.summary.json`
  - result:
    passed after adding the `level_is` observation surface, row-level expanded metadata, and direct
    TreeState selection/toggle updates in the Tree component policy layer.
- DataTable pagination disabled/invoke action-state mutation gate:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-default-pagination-collection-metadata.json`
  - extends the existing pagination metadata gate to assert Prev/Next `disabled_is` and
    `semantics_action_is(invoke)` before and after page changes
  - page 1 invariant:
    Prev is disabled and non-invokable; Next is enabled and invokable
  - final-page invariant:
    Prev is enabled and invokable; Next is disabled and non-invokable
  - diagnostics protocol support:
    `crates/fret-diag-protocol/src/lib.rs` and `crates/fret-diag-protocol/src/builder.rs`
  - runtime predicate and bundle export support:
    `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs` and
    `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`
  - synthetic fixture support:
    `crates/fret-ui/src/declarative/tests/fixtures/semantics_relations_v1.json` and
    `crates/fret-ui/src/declarative/tests/semantics_relations_harness.rs`
  - focused protocol gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_disabled_is_serializes_and_deserializes predicate_semantics_action_is_serializes_and_deserializes script_v2_roundtrip_ui_gallery_data_table_default_pagination_collection_metadata --no-fail-fast`
  - focused bootstrap gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics disabled_is_matches_semantics_disabled_flag semantics_action_is_matches_all_exported_action_flags semantics_node_exports_all_action_flags --no-fail-fast`
  - mechanism oracle gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-mechanism-harness semantics_value_state_actions_and_structured_metadata_are_queryable --no-fail-fast`
  - synthetic fixture gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_semantics_relations_match_oracles --no-fail-fast`
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-default-pagination-collection-metadata.json --dir target/fret-diag-data-table-default-pagination-collection-metadata-v2 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
  - current passing evidence:
    `target/fret-diag-data-table-default-pagination-collection-metadata-v2/sessions/1778795941512-76048/1778795949700/script.result.json`
  - current AI packet:
    `target/fret-diag-data-table-default-pagination-collection-metadata-v2/sessions/1778795941512-76048/1778795949700/ai.packet`
  - current share pack:
    `target/fret-diag-data-table-default-pagination-collection-metadata-v2/sessions/1778795941512-76048/share/1778795949700.zip`
  - result:
    passed after rebuilding `fretboard-dev`. No DataTable pagination component defect was
    reproduced; the fixed defect was a diagnostics/mechanism observability gap around disabled
    state, generic semantics actions, and complete action export.
- Retained Tree selected/invoke action-state mutation gate:
  `tools/diag-scripts/ui-gallery/tree/ui-gallery-tree-retained-hierarchy-semantics-toggle.json`
  - extends the existing retained Tree hierarchy script with row `selected_is`, `disabled_is`, and
    `semantics_action_is(invoke)` assertions
  - proves row `1000000` can become selected, detach while root is collapsed, reattach still
    selected after root expands, and then lose selection when row `2000000` is selected
  - script redirect:
    `tools/diag-scripts/ui-gallery-tree-retained-hierarchy-semantics-toggle.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-tree-retained/suite.json`
  - diagnostics catalog entry:
    `tools/diag-scripts/index.json` (`ui-gallery-tree-retained-hierarchy-semantics-toggle`)
  - focused script gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_tree_retained_hierarchy_semantics_toggle --no-fail-fast`
  - runtime command:
    `$env:FRET_UI_GALLERY_TREE_RETAINED='1'; target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/tree/ui-gallery-tree-retained-hierarchy-semantics-toggle.json --dir target/fret-diag-tree-retained-action-state-v1 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
  - current passing evidence:
    `target/fret-diag-tree-retained-action-state-v1/sessions/1778797137996-128620/1778797146847/script.result.json`
  - current AI packet:
    `target/fret-diag-tree-retained-action-state-v1/sessions/1778797137996-128620/1778797146847/ai.packet`
  - current share pack:
    `target/fret-diag-tree-retained-action-state-v1/sessions/1778797137996-128620/share/1778797146847.zip`
  - retained Tree suite command:
    `$env:FRET_UI_GALLERY_TREE_RETAINED='1'; target/dev-fast/fretboard-dev.exe diag suite ui-gallery-tree-retained --dir target/fret-diag-tree-retained-suite-after-action-state-v1 --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - current retained Tree suite summary:
    `target/fret-diag-tree-retained-suite-after-action-state-v1/sessions/1778797176571-37148/suite.summary.json`
  - result:
    passed; no retained Tree stale selected/invoke defect was reproduced. The remaining gap is
    dynamic disabled/invoke suppression on reused retained rows.
- Retained Tree dynamic disabled/focus/invoke suppression gate:
  `tools/diag-scripts/ui-gallery/tree/ui-gallery-tree-retained-hierarchy-semantics-toggle.json`
  - extends the same retained Tree lifecycle with diagnostics-only disabled-state mutation for row
    `2000000`
  - proves `disabled_is=true`, `semantics_action_is(focus)=false`, and
    `semantics_action_is(invoke)=false` after toggling the row disabled, proves clicking that
    disabled row does not move selection away from row `1000000`, then proves re-enabling restores
    `disabled_is=false`, `semantics_action_is(invoke)=true`, and click selection
  - diagnostics control:
    `apps/fret-ui-gallery/src/ui/previews/gallery/data/tree_torture.rs`
    (`ui-gallery-tree-toggle-target-disabled`)
  - focused compile/check gates:
    `cargo fmt --package fret-ui-gallery`,
    `cargo check --profile dev-fast -p fret-ui-gallery --features gallery-dev`, and
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - focused script gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_tree_retained_hierarchy_semantics_toggle --no-fail-fast`
  - failed keyboard-route probes:
    `target/fret-diag-tree-retained-disabled-keyboard-v1/sessions/1778799162763-12836/1778799170889/script.result.json`
    and
    `target/fret-diag-tree-retained-disabled-keyboard-v2/sessions/1778799309973-128636/1778799318772/script.result.json`
  - keyboard-route conclusion:
    disabled Tree rows correctly lose focus/invoke action, so this surface is not a valid
    disabled-but-focusable Enter/Space activation target
  - runtime command:
    `$env:FRET_UI_GALLERY_TREE_RETAINED='1'; target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/tree/ui-gallery-tree-retained-hierarchy-semantics-toggle.json --dir target/fret-diag-tree-retained-disabled-focus-action-v1 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
  - current passing evidence:
    `target/fret-diag-tree-retained-disabled-focus-action-v1/sessions/1778799548520-155404/1778799557705/script.result.json`
  - current AI packet:
    `target/fret-diag-tree-retained-disabled-focus-action-v1/sessions/1778799548520-155404/1778799557705/ai.packet`
  - current share pack:
    `target/fret-diag-tree-retained-disabled-focus-action-v1/sessions/1778799548520-155404/share/1778799557705.zip`
  - retained Tree suite command:
    `$env:FRET_UI_GALLERY_TREE_RETAINED='1'; target/dev-fast/fretboard-dev.exe diag suite ui-gallery-tree-retained --dir target/fret-diag-tree-retained-suite-after-disabled-focus-action-v1 --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - current retained Tree suite summary:
    `target/fret-diag-tree-retained-suite-after-disabled-focus-action-v1/sessions/1778799592687-38760/suite.summary.json`
  - result:
    passed; no retained Tree stale disabled/focus/invoke defect was reproduced. The next uncovered
    route is keyboard/action activation suppression on a focusable-disabled recipe/primitive
    surface rather than Tree disabled rows.
- Accordion focusable-disabled keyboard/action suppression gate:
  `tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-focusable-disabled-keyboard-suppression.json`
  - proves the Radix-style open non-collapsible trigger outcome: `disabled_is=true`,
    `expanded_is=true`, `semantics_action_is(focus)=true`,
    `semantics_action_is(invoke)=false`, direct focus succeeds, and Enter/Space do not collapse the
    item
  - UI Gallery snippet:
    `apps/fret-ui-gallery/src/ui/snippets/accordion/focusable_disabled.rs`
  - recipe fix:
    `ecosystem/fret-ui-shadcn/src/accordion.rs`
  - focused recipe gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib accordion_trigger_open_non_collapsible_is_aria_disabled --no-fail-fast`
  - focused recipe result:
    passed; Nextest run id `8bcfd907-e08a-4c65-a183-4ed6c8c4ca5f`
  - first failed runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-focusable-disabled-keyboard-suppression.json --dir target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v1 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
  - first failed evidence:
    `target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v1/sessions/1778801842689-150456/1778801848335/script.result.json`
  - failure slice:
    `target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v1/sessions/1778801842689-150456/1778801848335/slice.ui-gallery-accordion-focusable-disabled-trigger.json`
    (`actions.focus=true`, `flags.disabled=true`, `flags.expanded=true`, `bounds.w=0.0`)
  - runtime command after fix:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-focusable-disabled-keyboard-suppression.json --dir target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v2 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
  - current passing evidence:
    `target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v2/sessions/1778804970965-48396/1778804975621/script.result.json`
  - current AI packet:
    `target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v2/sessions/1778804970965-48396/1778804975621/ai.packet`
  - current share pack:
    `target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v2/sessions/1778804970965-48396/share/1778804975621.zip`
  - result:
    fixed and promoted. The runtime harness found a real shadcn Accordion recipe layout defect
    before passing after the wrapper width fix.
- Pressable focusable-disabled key-activation mechanism fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/pressable_key_activation_v1.json`
  - covers Enter+Space, Enter-only, `PressableKeyActivation::None`, focusable-disabled semantics
    with `disabled=true`/`focus=true`/`invoke=false`, and fully disabled Pressable semantics/action
    outcomes
  - thin harness:
    `crates/fret-ui/src/declarative/tests/pressable_key_activation_harness.rs`
  - mechanism change:
    `crates/fret-ui/src/element.rs` (`PressableKeyActivation::None`)
  - policy consumer:
    `ecosystem/fret-ui-kit/src/primitives/accordion.rs`
    (`apply_accordion_trigger_aria_disabled`)
  - focused mechanism gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui --lib mechanism_harness_pressable_key_activation_matches_oracles --no-fail-fast`
  - focused mechanism result:
    passed; Nextest run id `3e2def17-044d-4583-9796-f625ee4367af`
  - focused primitive gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit --lib apply_accordion_trigger_aria_disabled_suppresses_keyboard_activation_on_pressable --no-fail-fast`
  - focused primitive result:
    passed; Nextest run id `47d0e780-77bf-4b90-9d0e-9ec1bc481506`
  - post-mechanism runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-focusable-disabled-keyboard-suppression.json --dir target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v3 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
  - current passing evidence:
    `target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v3/sessions/1778806281798-4932/1778806286335/script.result.json`
  - current AI packet:
    `target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v3/sessions/1778806281798-4932/1778806286335/ai.packet`
  - current share pack:
    `target/fret-diag-accordion-focusable-disabled-keyboard-suppression-v3/sessions/1778806281798-4932/share/1778806286335.zip`
  - result:
    fixed and promoted. The focusable-disabled keyboard suppression axis is now available at the
    core Pressable mechanism layer and consumed by Accordion.
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
- Combobox responsive resize-reposition gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-resize-open-placement.json`
  - runtime assertions:
    opens the responsive Combobox popover, proves first-open bottom placement, resizes the window
    while open, proves the input/content/listbox remain mounted, waits for a fresh post-resize
    `anchored_panel` trace with `preferred_side=bottom` and `side_offset_px=4`, allows collision
    flip, asserts the content stays in-window, asserts the top-flip gap
    `content.bottom - trigger.top = -4px`, checks the documented desktop content/trigger width
    delta, preserves `controls`/`labelled_by` relation wiring, and waits for stable bounds before
    capture.
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-combobox/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-resize-open-placement.json --dir target/fret-diag-combobox-responsive-resize-open-placement-v4 --session-auto --pack --ai-packet --include-screenshots --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778847566014`.
  - runtime evidence:
    `target/fret-diag-combobox-responsive-resize-open-placement-v4/sessions/1778847561470-88320/1778847566014/script.result.json`
  - AI packet:
    `target/fret-diag-combobox-responsive-resize-open-placement-v4/sessions/1778847561470-88320/1778847566014/ai.packet`
  - packed evidence:
    `target/fret-diag-combobox-responsive-resize-open-placement-v4/sessions/1778847561470-88320/share/1778847566014.zip`
  - script roundtrip:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_responsive_resize_open_placement --no-fail-fast`
  - script roundtrip result:
    passed, 1 test; Nextest run id `f8d2693a-e141-4dfc-a171-169b051971de`.
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
- DropdownMenu disabled-but-focusable keyboard suppression gate:
  `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json`
  - suite membership:
    `tools/diag-scripts/suites/fret-mechanism-harness-overlay-focus/suite.json`
  - focused shadcn gate:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib dropdown_menu_disabled_focusable -- --nocapture`
  - focused shadcn result:
    passed, 2 tests.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json --dir target/fret-diag-dropdown-menu-focusable-disabled-v3 --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778816892364`.
  - runtime evidence:
    `target/fret-diag-dropdown-menu-focusable-disabled-v3/sessions/1778816884031-66372/script.result.json`
  - focused bundle:
    `target/fret-diag-dropdown-menu-focusable-disabled-v3/sessions/1778816884031-66372/1778816944901-ui-gallery-dropdown-menu-focusable-disabled-focused`
  - final suppression bundle:
    `target/fret-diag-dropdown-menu-focusable-disabled-v3/sessions/1778816884031-66372/1778816947251-ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression`
  - registry gate:
    `python tools/check_diag_scripts_registry.py` passed after promotion into the overlay/focus
    suite.
- Command disabled-but-focusable active-descendant keyboard suppression gate:
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-palette-disabled-focusable-keyboard-suppression.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-command/suite.json` and
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`
  - focused shadcn gate:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib disabled_item -- --nocapture`
  - focused shadcn result:
    passed, 2 tests.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-palette-disabled-focusable-keyboard-suppression.json --dir target/fret-diag-command-disabled-focusable-keyboard-suppression-v2 --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778824893798`.
  - runtime evidence:
    `target/fret-diag-command-disabled-focusable-keyboard-suppression-v2/sessions/1778824882684-64832/script.result.json`
  - active-descendant bundle:
    `target/fret-diag-command-disabled-focusable-keyboard-suppression-v2/sessions/1778824882684-64832/1778824950725-ui-gallery-command-palette-disabled-focusable-active-descendant`
  - keyboard-suppression bundle:
    `target/fret-diag-command-disabled-focusable-keyboard-suppression-v2/sessions/1778824882684-64832/1778824952379-ui-gallery-command-palette-disabled-focusable-keyboard-suppression`
  - full Command suite:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-command --dir target/fret-diag-command-suite-disabled-focusable-v7 --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - suite result:
    passed; 17 scripts, zero lint errors, zero reason codes.
  - suite evidence:
    `target/fret-diag-command-suite-disabled-focusable-v7/sessions/1778825524424-52724/suite.summary.json`
- Retained/windowed active-descendant action-state mutation gate:
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-retained-active-descendant-action-state.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-command/suite.json`,
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`, and
    `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`
  - synthetic mechanism gate:
    `cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_combobox_active_descendant_interaction_matches_oracles -- --nocapture`
  - synthetic result:
    passed, 1 test.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - build result:
    passed.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-retained-active-descendant-action-state.json --dir target/fret-diag-command-retained-active-descendant-action-state-v3 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778832051741`.
  - runtime evidence:
    `target/fret-diag-command-retained-active-descendant-action-state-v3/sessions/1778832029259-77072/script.result.json`
  - detached bundle:
    `target/fret-diag-command-retained-active-descendant-action-state-v3/sessions/1778832029259-77072/1778832142314-ui-gallery-command-retained-active-descendant-detached`
  - final action-state bundle:
    `target/fret-diag-command-retained-active-descendant-action-state-v3/sessions/1778832029259-77072/1778832143417-ui-gallery-command-retained-active-descendant-action-state`
  - bundle lint:
    both captured bundles passed `target/dev-fast/fretboard-dev.exe diag lint ... --json` with
    `error_issues=0` and `warning_issues=0`.
  - full Command suite:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-command --dir target/fret-diag-command-suite-retained-active-descendant-v2 --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - suite result:
    passed; 18 scripts, zero reason codes.
  - suite evidence:
    `target/fret-diag-command-suite-retained-active-descendant-v2/sessions/1778831271030-63304/suite.summary.json`
- Semantics relation-edge detach/reattach fixture and diagnostics predicates:
  `crates/fret-ui/src/declarative/tests/fixtures/semantics_relations_v1.json`
  - fixture case:
    `relation-targets-detach-reattach-clear-stale-edges`
  - mechanism gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_semantics_relations_match_oracles`
  - mechanism result:
    passed, 1 test.
  - protocol serialization gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_semantics_relation_serializes_and_deserializes`
  - protocol serialization result:
    passed, 1 test.
  - script roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v1_roundtrip_semantics_relation_predicates`
  - script roundtrip result:
    passed, 1 test.
  - builder helper gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol builder_v2_semantics_relation_predicates_serialize`
  - builder helper result:
    passed, 1 test.
  - bootstrap predicate evaluator gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics semantics_relation_predicates_match_semantics_edges`
  - bootstrap predicate result:
    passed, 1 test.
  - implementation anchors:
    `crates/fret-diag-protocol/src/lib.rs`,
    `crates/fret-diag-protocol/src/builder.rs`,
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`,
    `crates/fret-diag-protocol/tests/builder_smoke.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
    `crates/fret-ui/src/declarative/tests/semantics_relations_harness.rs`
- Cross-root Select relation runtime gate:
  `tools/diag-scripts/ui-gallery/select/ui-gallery-select-commit-and-label-update.json`
  - runtime assertions:
    trigger `controls` listbox after open, listbox `labelled_by` trigger after open, trigger
    `controls` empty after commit/close, and trigger `controls` restored after reopen.
  - focused bootstrap relation endpoint gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics semantics_relation_predicates_match_semantics_edges semantics_relation_includes_can_cross_scope_roots semantics_relation_includes_can_cross_modal_barrier_to_underlay_source`
  - focused bootstrap result:
    passed, 3 tests.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - build result:
    passed.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/select/ui-gallery-select-commit-and-label-update.json --dir target/fret-diag-select-relation-edge-runtime-v8 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778839948330`.
  - runtime evidence:
    `target/fret-diag-select-relation-edge-runtime-v8/sessions/1778839940868-81380/script.result.json`
  - AI packet:
    `target/fret-diag-select-relation-edge-runtime-v8/sessions/1778839940868-81380/1778839948330/ai.packet`
  - packed evidence:
    `target/fret-diag-select-relation-edge-runtime-v8/sessions/1778839940868-81380/share/1778839948330.zip`
  - protocol companion gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_semantics_relation_serializes_and_deserializes script_v1_roundtrip_semantics_relation_predicates builder_v2_semantics_relation_predicates_serialize --no-fail-fast`
  - protocol companion result:
    passed, 3 tests.
  - Select content selector gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib select_content_test_id_stamps_listbox_without_renaming_viewport select_test_id_prefix_stamps_listbox_items_and_scroll_viewport`
  - Select content selector result:
    passed, 2 tests.
  - implementation anchors:
    `ecosystem/fret-bootstrap/src/ui_diagnostics/selector.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
    `ecosystem/fret-ui-shadcn/src/select.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/select/diag_surface.rs`,
    `tools/diag-scripts/ui-gallery/select/ui-gallery-select-commit-and-label-update.json`
- Select item-aligned resize-close placement policy gate:
  `tools/diag-scripts/ui-gallery/select/ui-gallery-select-demo-open-layout.json`
  - runtime assertions:
    first-open placed-rect ownership, listbox containment and bounds stability, trigger `controls`
    listbox, listbox `labelled_by` trigger, then window resize closes the item-positioned popup,
    leaves the trigger stable in the viewport, and clears trigger `controls`.
  - policy anchor:
    `ecosystem/fret-ui-kit/src/primitives/select.rs` (`modal_select_request` opts into
    `close_on_window_resize`).
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/select/ui-gallery-select-demo-open-layout.json --dir target/fret-diag-select-demo-open-layout-resize-closes-v1 --session-auto --pack --ai-packet --include-screenshots --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778846116295`.
  - runtime evidence:
    `target/fret-diag-select-demo-open-layout-resize-closes-v1/sessions/1778846111779-85184/1778846116295/script.result.json`
  - AI packet:
    `target/fret-diag-select-demo-open-layout-resize-closes-v1/sessions/1778846111779-85184/1778846116295/ai.packet`
  - packed evidence:
    `target/fret-diag-select-demo-open-layout-resize-closes-v1/sessions/1778846111779-85184/share/1778846116295.zip`
  - focused policy gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit modal_select_request_sets_default_root_name --no-fail-fast`
  - focused policy result:
    passed, 1 test; Nextest run id `fd96f104-b2ba-4fce-9390-ebd8ef9667aa`.
  - script roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_select_demo_open_layout --no-fail-fast`
  - script roundtrip result:
    passed, 1 test; Nextest run id `228efd90-671a-42f6-81bf-bd9c8b1c1397`.
- Diagnostics authoring long-page click visibility gate:
  `tools/check_diag_scripts_registry.py`
  - scope:
    promoted `ui-gallery-combobox` suite scripts, content targets with
    `ui-gallery-combobox-*` test ids.
  - invariant:
    long-page content targets may not use plain `click`; `click_stable` requires a prior
    `scroll_into_view(require_fully_within_window=true)` or `bounds_within_window` guard for the
    same target.
  - found:
    495 unsafe content-click patterns across the promoted registry, with 15 in
    `ui-gallery-combobox`; the Combobox family is now cleared to zero and guarded.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
  - script roundtrip gate:
    `cargo nextest run -p fret-diag-protocol --test script_json_roundtrip`
  - script roundtrip result:
    passed, 103 tests; Nextest run id `699852bd-a308-4657-bcd1-3f87f9243d3b`.
  - build gate after stale diagnostics binary discovery:
    `cargo build -p fretboard-dev -p fret-ui-gallery`
  - build result:
    passed.
  - focused responsive resize rerun:
    `target\debug\fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-resize-open-placement.json --dir target/fret-diag-combobox-responsive-resize-authoring-lint-v2 --session-auto --pack --ai-packet --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  - focused runtime result:
    passed; run id `1778851448831`.
  - focused runtime evidence:
    `target/fret-diag-combobox-responsive-resize-authoring-lint-v2/sessions/1778851446231-91900/1778851448831/script.result.json`
  - focused AI packet:
    `target/fret-diag-combobox-responsive-resize-authoring-lint-v2/sessions/1778851446231-91900/1778851448831/ai.packet`
  - focused packed evidence:
    `target/fret-diag-combobox-responsive-resize-authoring-lint-v2/sessions/1778851446231-91900/share/1778851448831.zip`
  - family suite:
    `target\debug\fretboard-dev.exe diag suite ui-gallery-combobox --dir target/fret-diag-combobox-suite-authoring-lint-v2 --session-auto --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  - family suite result:
    passed, 23 scripts.
  - family suite evidence:
    `target/fret-diag-combobox-suite-authoring-lint-v2/sessions/1778851461244-45400/suite.summary.json`
  - proof:
    `status=passed`, `stage_counts.passed=23`, `reason_code_counts={}`,
    `scripts_with_evidence=23`, and `focus_mismatch_total=0`.
- Select active-descendant/view-cache notification and scroll lint classification:
  `tools/diag-scripts/ui-gallery/select/ui-gallery-select-roving-skips-disabled-orange.json`
  - invariant:
    keyboard navigation after pointer-open must update the active descendant in the real UI Gallery
    semantics snapshot even when Select active-row state is stored outside the element tree and the
    view cache can otherwise be reused.
  - owning fixes:
    `ecosystem/fret-ui-kit/src/primitives/select.rs`,
    `ecosystem/fret-ui-shadcn/src/select.rs`,
    `crates/fret-diag/src/stats/wheel_scroll.rs`,
    `crates/fret-diag/src/stats/wheel_scroll_streaming/checks.rs`,
    `crates/fret-diag/src/stats/wheel_scroll_streaming/tests.rs`,
    and `crates/fret-diag/src/lint.rs`.
  - focused headless gate:
    `cargo test -p fret-ui-kit --lib content_arrow -- --nocapture`
  - focused headless result:
    passed, 3 tests.
  - focused recipe gate:
    `cargo test -p fret-ui-shadcn --lib select_grouped_pointer_open_arrow_down_moves_active_descendant -- --nocapture`
  - focused recipe result:
    passed.
  - build gate:
    `cargo build -p fretboard-dev -p fret-ui-gallery`
  - build result:
    passed.
  - original failing runtime gate:
    `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\select\ui-gallery-select-roving-skips-disabled-orange.json --dir target\fret-diag-select-roving-after-notify-v1 --session-auto --pack --ai-packet --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  - original failing runtime result:
    passed.
  - original failing runtime evidence:
    `target/fret-diag-select-roving-after-notify-v1/sessions/1778870575466-4348/1778870578695/script.result.json`
  - original failing AI packet:
    `target/fret-diag-select-roving-after-notify-v1/sessions/1778870575466-4348/1778870578695/ai.packet`
  - original failing packed evidence:
    `target/fret-diag-select-roving-after-notify-v1/sessions/1778870575466-4348/share/1778870578695.zip`
  - wheel-scroll classification gate:
    `cargo test -p fret-diag wheel_scroll_hit_changes -- --nocapture`
  - wheel-scroll classification result:
    passed, 5 tests.
  - active-descendant lint classification gate:
    `cargo test -p fret-diag lint_downgrades_scrollable_active_descendant_out_of_window_to_warning -- --nocapture`
  - active-descendant lint classification result:
    passed.
  - family suite after merging local `main`:
    `target\debug\fretboard-dev.exe diag suite ui-gallery-select --dir target\fret-diag-select-suite-post-main-merge-v1 --session-auto --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  - family suite result:
    passed, 14 scripts.
  - family suite evidence:
    `target/fret-diag-select-suite-post-main-merge-v1/sessions/1778876429637-35368/suite.summary.json`
  - companion authoring gate:
    `tools/check_diag_scripts_registry.py` now includes `ui-gallery-select` in the strict long-page
    click-visibility suite set, so promoted Select scripts cannot regress to plain content clicks
    or unguarded `click_stable` on `ui-gallery-select-*` content targets.
- Select scrollable placement boundary baseline:
  `tools/diag-scripts/ui-gallery/select/ui-gallery-select-scrollable-placement-boundary.json`
  - invariant:
    the scrollable Select docs surface must expose stable trigger/listbox diagnostics ids, open a
    long-list item-aligned popup inside a constrained viewport, emit a placed-rect trace, keep the
    listbox inside the window, bound listbox size, preserve trigger/listbox relations, and leave
    layout sidecar evidence for clipping-boundary inspection.
  - implementation anchors:
    `apps/fret-ui-gallery/src/ui/snippets/select/scrollable.rs`,
    `tools/diag-scripts/suites/ui-gallery-select/suite.json`,
    `tools/diag-scripts/index.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - script roundtrip gate:
    `cargo nextest run -p fret-diag-protocol script_v2_roundtrip_ui_gallery_select_scrollable_placement_boundary --no-fail-fast`
  - script roundtrip result:
    passed, 1 test; Nextest run id `656d7088-0176-4aa6-a1d2-162e5be92930`.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
  - build gate:
    `cargo build -p fretboard-dev -p fret-ui-gallery`
  - build result:
    passed.
  - focused runtime command:
    `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\select\ui-gallery-select-scrollable-placement-boundary.json --dir target\fret-diag-select-scrollable-placement-boundary-v4 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  - focused runtime result:
    passed.
  - focused runtime evidence:
    `target/fret-diag-select-scrollable-placement-boundary-v4/sessions/1778877971968-93800/share/1778877974456.zip`
  - focused AI packet:
    `target/fret-diag-select-scrollable-placement-boundary-v4/sessions/1778877971968-93800/1778877974456/ai.packet`
  - family suite:
    `target\debug\fretboard-dev.exe diag suite ui-gallery-select --dir target\fret-diag-select-suite-scrollable-placement-boundary-v1 --session-auto --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  - family suite result:
    passed.
  - family suite evidence:
    `target/fret-diag-select-suite-scrollable-placement-boundary-v1/sessions/1778878050419-40260/suite.summary.json`
- Combobox scroll-container/RTL placement ownership gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-placement-ownership-scroll-rtl.json`
  - invariant:
    an RTL Combobox trigger inside a clipped ScrollArea must place its popover on the overlay root,
    not under the scroll viewport clip; the content and `Release Ready` option intentionally extend
    below the ScrollArea viewport, and clicking that option must still commit selection.
  - implementation anchors:
    `apps/fret-ui-gallery/src/ui/snippets/combobox/placement_ownership.rs`,
    `apps/fret-ui-gallery/src/ui/pages/combobox.rs`,
    `tools/diag-scripts/suites/ui-gallery-combobox/suite.json`,
    `tools/diag-scripts/index.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - first failed evidence:
    `target/fret-diag-combobox-placement-ownership-scroll-rtl-v1/sessions/1778879398702-31444/1778879401346/script.result.json`
    showed a valid trace with `chosen_side=top`, `flipped=true`, and only
    `preferred_available_main_px=57.33`; the failure was a harness setup issue, not a mechanism
    defect.
  - script roundtrip gate:
    `cargo nextest run -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_placement_ownership_scroll_rtl --no-fail-fast`
  - script roundtrip result:
    passed, 1 test; Nextest run id `487c4520-a050-4ba1-aa1b-3e6d93e591a3`.
  - build gate:
    `cargo build -p fretboard-dev -p fret-ui-gallery`
  - build result:
    passed.
  - focused runtime command:
    `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-placement-ownership-scroll-rtl.json --dir target\fret-diag-combobox-placement-ownership-scroll-rtl-v3 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  - focused runtime result:
    passed.
  - focused runtime evidence:
    `target/fret-diag-combobox-placement-ownership-scroll-rtl-v3/sessions/1778879722585-104312/script.result.json`
  - focused AI packet:
    `target/fret-diag-combobox-placement-ownership-scroll-rtl-v3/sessions/1778879722585-104312/1778879725145/ai.packet`
  - focused packed evidence:
    `target/fret-diag-combobox-placement-ownership-scroll-rtl-v3/sessions/1778879722585-104312/share/1778879725145.zip`
  - family suite:
    `target\debug\fretboard-dev.exe diag suite ui-gallery-combobox --dir target\fret-diag-combobox-suite-placement-ownership-v1 --session-auto --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  - family suite result:
    passed, 24 scripts.
  - family suite evidence:
    `target/fret-diag-combobox-suite-placement-ownership-v1/sessions/1778879747388-60292/suite.summary.json`
- Dialog nested Combobox modal-boundary ownership gate:
  `tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-nested-combobox-modal-boundary.json`
  - invariant:
    a Combobox opened inside a modal Dialog must remain selectable while the Dialog modal/focus
    barrier is active, even when the Combobox content lives in an overlay root above the barrier
    rather than as a descendant of the barrier root. The gate also proves placement, relation
    wiring, selected state, screenshot, bundle, layout sidecar evidence, and final barrier cleanup.
  - implementation anchors:
    `ecosystem/fret-bootstrap/src/ui_diagnostics/selector.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/selector_resolution_trace_recording.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/dialog/nested_combobox.rs`,
    `apps/fret-ui-gallery/src/ui/pages/dialog.rs`,
    `tools/diag-scripts/suites/fret-mechanism-harness-overlay-focus/suite.json`,
    `tools/diag-scripts/index.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - focused diagnostics selector gate:
    `cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics" --lib wait_until_selector_trace_reports_modal_barrier_filtering`
  - focused diagnostics selector result:
    passed, 1 test.
  - full diagnostics feature lib gate:
    `cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics" --lib --no-fail-fast`
  - full diagnostics feature lib result:
    passed, 154 tests.
  - runtime command:
    `target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-nested-combobox-modal-boundary.json --dir target/fret-diag-dialog-nested-combobox-modal-boundary-v4 --session-auto --pack --pack-schema2-only --json --exit-after-run --launch cargo run -p fret-ui-gallery`
  - runtime result:
    passed; run id `1778885770482`.
  - runtime evidence:
    `target/fret-diag-dialog-nested-combobox-modal-boundary-v4/sessions/1778885765520-86120/script.result.json`
  - schema2 bundle:
    `target/fret-diag-dialog-nested-combobox-modal-boundary-v4/sessions/1778885765520-86120/1778885771913-ui-gallery-dialog-nested-combobox-modal-boundary/bundle.schema2.json`
  - packed evidence:
    `target/fret-diag-dialog-nested-combobox-modal-boundary-v4/sessions/1778885765520-86120/share/1778885770482.zip`
- Cross-root anchored coordinate/root-boundary policy gate:
  `crates/fret-ui/src/declarative/tests/fixtures/anchored_cross_root_coordinate_v1.json`
  - invariant:
    core `AnchoredProps` must resolve anchors across render roots and clamp/flip against the
    current overlay/root boundary, while shadcn anchored overlay recipes should derive collision
    boundaries from the owner render root rather than the OS window/environment viewport.
  - implementation anchors:
    `crates/fret-ui/src/declarative/tests/anchored_cross_root_coordinate_harness.rs`,
    `ecosystem/fret-ui-kit/src/overlay.rs`,
    `ecosystem/fret-ui-shadcn/src/popover.rs`,
    `ecosystem/fret-ui-shadcn/src/select.rs`,
    `ecosystem/fret-ui-shadcn/src/tooltip.rs`,
    `ecosystem/fret-ui-shadcn/src/hover_card.rs`,
    `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`,
    `ecosystem/fret-ui-shadcn/src/context_menu.rs`, and
    `ecosystem/fret-ui-shadcn/src/menubar.rs`.
  - JSON fixture result:
    `python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/anchored_cross_root_coordinate_v1.json`
    passed.
  - synthetic gate:
    `cargo test -p fret-ui --lib mechanism_harness_anchored_cross_root_coordinate_matches_oracles -- --nocapture`
  - synthetic result:
    passed, 1 test.
  - UI kit focused gate:
    `cargo nextest run -p fret-ui-kit outer_bounds_with_window_margin --no-fail-fast`
  - UI kit focused result:
    passed, 2 tests; Nextest run id `18deeb53-0fd8-4b25-8df3-af61212780e9`.
  - shadcn compile/placement smoke:
    `$env:CARGO_BUILD_JOBS='1'; cargo test -p fret-ui-shadcn --lib popover_first_open_placement_size_prefers_explicit_hint -- --nocapture`
  - shadcn compile/placement smoke result:
    passed, 1 test.
  - representative shadcn focused gates:
    `hover_card_anchor_override_uses_anchor_bounds_for_placement`,
    `tooltip_anchor_override_uses_anchor_bounds_for_placement`,
    `dropdown_menu_portal_escapes_overflow_clip_ancestor`,
    `context_menu_submenu_keyboard_open_transfers_focus_and_arrow_left_restores_focus`, and
    `menubar_submenu_opens_on_arrow_right_and_closes_on_arrow_left_restoring_focus`.
  - representative shadcn focused result:
    all passed.
- Runtime multi-viewport Combobox root-boundary gate:
  `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-multi-viewport-combobox-placement.json`
  - invariant:
    a Combobox trigger inside a Resizable panel viewport root must derive overlay collision
    boundaries from the source element's nearest layout viewport root, not from the owner root or
    OS window. The fixture places the trigger near the bottom of the panel while the OS window still
    has room below; correct behavior flips the popover to the top.
  - implementation anchors:
    `crates/fret-ui/src/elements/runtime.rs`,
    `crates/fret-ui/src/elements/cx.rs`,
    `crates/fret-ui/src/elements/queries.rs`,
    `crates/fret-ui/src/tree/layout/entrypoints.rs`,
    `crates/fret-ui/src/declarative/tests/layout/viewport_roots.rs`,
    `ecosystem/fret-ui-kit/src/overlay.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/resizable/multi_viewport_combobox.rs`,
    `apps/fret-ui-gallery/src/ui/pages/resizable.rs`,
    `tools/diag-scripts/suites/fret-mechanism-harness-overlay-focus/suite.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - before runtime evidence:
    `.fret/diag/runs/ui-gallery-resizable-multi-viewport-combobox-placement-before/script.result.json`
  - before result:
    failed at step 14 with `wait_overlay_placement_trace_timeout`; trace showed
    `chosen_side=bottom` and `outer_collision=900x1000@0,0`.
  - before packed evidence:
    `.fret/diag/runs/ui-gallery-resizable-multi-viewport-combobox-placement-before/share/1778902294871.zip`
  - focused mechanism gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui element_root_bounds_cache_rebuilds_on_view_cache_hit_after_viewport_move element_root_bounds_cache_rebuilds_when_element_moves_between_viewport_roots element_root_bounds_cache_uses_nearest_nested_viewport_root element_root_bounds_cache_uses_nearest_viewport_when_owner_root_differs viewport_root_bounds_for_descendant_elements_track_the_panel_bounds --no-fail-fast`
  - focused mechanism result:
    passed, 5 tests; Nextest run id `045558b5-2637-4b15-a41f-55d7399d5ad2`.
  - UI kit focused gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit outer_bounds_with_window_margin_for_root_uses_current_context_bounds popper_layout_for_element_returns_arrow_layout_when_configured --no-fail-fast`
  - UI kit focused result:
    passed, 2 tests; Nextest run id `819d431c-1316-431e-95b8-58dc6e962904`.
  - protocol roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_resizable_multi_viewport_combobox_placement --no-fail-fast`
  - protocol roundtrip result:
    passed, 1 test; Nextest run id `6f146a5c-1f46-4982-b37d-4722a3909e4d`.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-multi-viewport-combobox-placement.json --dir .fret/diag/runs/ui-gallery-resizable-multi-viewport-combobox-placement-final2 --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778905890519`.
  - runtime evidence:
    `.fret/diag/runs/ui-gallery-resizable-multi-viewport-combobox-placement-final2/script.result.json`
  - packed evidence:
    `.fret/diag/runs/ui-gallery-resizable-multi-viewport-combobox-placement-final2/share/1778905890519.zip`
  - structured trace proof:
    `chosen_side=top`, `preferred_fits_without_main_clamp=false`, and
    `outer_collision=336x378@514.67,468.67`.
- Non-modal overlay underlay activation-status gates:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-click-through-outside-press-focus-underlay.json`
  and
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dropdown-nonmodal-outside-press-focus-underlay.json`
  - invariant:
    outside press on a default-compatible non-modal overlay must dismiss/close the overlay, move
    focus to the underlay probe, and deliver the click to the underlay activation handler. Focus and
    dismiss status are not strong enough proxy signals for click-through correctness.
  - implementation anchors:
    `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/overlay/widgets.rs`,
    `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/overlay/flags.rs`,
    `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-click-through-outside-press-focus-underlay.json`,
    `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dropdown-nonmodal-outside-press-focus-underlay.json`,
    and `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - finding:
    found a harness weakness, not a new mechanism defect. The previous runtime gates only proved
    focus/dismiss outcomes and could miss an outside-press policy regression where the underlay
    gained focus but the activation handler did not run.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - Popover runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-click-through-outside-press-focus-underlay.json --dir .fret/diag/runs/ui-gallery-popover-click-through-outside-press-focus-underlay-activation --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - Popover runtime result:
    passed; run id `1778906489806`.
  - Popover packed evidence:
    `.fret/diag/runs/ui-gallery-popover-click-through-outside-press-focus-underlay-activation/share/1778906489806.zip`
  - DropdownMenu runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-dropdown-nonmodal-outside-press-focus-underlay.json --dir .fret/diag/runs/ui-gallery-dropdown-nonmodal-outside-press-focus-underlay-activation --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - DropdownMenu runtime result:
    passed; run id `1778906522304`.
  - DropdownMenu packed evidence:
    `.fret/diag/runs/ui-gallery-dropdown-nonmodal-outside-press-focus-underlay-activation/share/1778906522304.zip`
  - protocol roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_popover_click_through_outside_press_focus_underlay script_v2_roundtrip_ui_gallery_dropdown_nonmodal_outside_press_focus_underlay --no-fail-fast`
  - protocol roundtrip result:
    passed, 2 tests; Nextest run id `c563620f-80b3-4933-9da6-48d657c68a38`.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
- Switch read-only action-state runtime gate:
  `tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-read-only-action-state.json`
  - invariant:
    a read-only Switch must remain observable and focusable, publish `read_only=true`, keep
    `disabled=false`, suppress `invoke`, and reject pointer, associated-label, Space, and Enter
    activation attempts without changing checked state.
  - implementation anchors:
    `crates/fret-diag-protocol/src/lib.rs`,
    `crates/fret-diag-protocol/src/builder.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`,
    `ecosystem/fret-ui-shadcn/src/switch.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/switch/read_only.rs`,
    `apps/fret-ui-gallery/src/ui/pages/switch.rs`,
    `tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-read-only-action-state.json`,
    and `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - finding:
    found a real shadcn recipe semantics defect. `Switch::read_only(true)` already blocked pointer
    mutation, but its semantics snapshot still exposed `actions.invoke=true`.
  - before focused command:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib switch_read_only_exposes_semantics_and_blocks_activation -- --nocapture`
  - before focused result:
    failed with `read-only switches must not expose invoke semantics`.
  - focused recipe command after fix:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib switch_read_only_exposes_semantics_and_blocks_activation -- --nocapture`
  - focused recipe result:
    passed, 1 test.
  - protocol gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_read_only_is_serializes_and_deserializes predicate_disabled_is_serializes_and_deserializes predicate_semantics_action_is_serializes_and_deserializes script_v2_roundtrip_ui_gallery_switch_read_only_action_state --no-fail-fast`
  - protocol result:
    passed, 4 tests; Nextest run id `c521d101-11fc-4c8e-b094-47bea4a5b822`.
  - bootstrap predicate gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics read_only_is_matches_semantics_read_only_flag disabled_is_matches_semantics_disabled_flag semantics_action_is_matches_all_exported_action_flags --no-fail-fast`
  - bootstrap predicate result:
    passed, 3 tests; Nextest run id `6b8a60cd-00a7-44ab-93bc-9e8a5cce6fce`.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-read-only-action-state.json --dir .fret/diag/runs/ui-gallery-switch-read-only-action-state-f112 --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778909364811`.
  - runtime evidence:
    `.fret/diag/runs/ui-gallery-switch-read-only-action-state-f112/script.result.json`
  - packed evidence:
    `.fret/diag/runs/ui-gallery-switch-read-only-action-state-f112/share/1778909364811.zip`
- Switch dynamic read-only action-state runtime gate:
  `tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-read-only-dynamic-action-state.json`
  - invariant:
    read-only policy changes must refresh a non-list Switch's `read_only` and `invoke` semantics
    across frames, while preserving focusability and checked-state behavior.
  - implementation anchors:
    `apps/fret-ui-gallery/src/ui/snippets/switch/read_only.rs`,
    `ecosystem/fret-ui-shadcn/src/switch.rs`,
    `tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-read-only-dynamic-action-state.json`,
    `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - finding:
    no new runtime defect reproduced after F112. The first focused test draft exposed a test
    modeling issue: changing a model without rerendering the declarative root cannot prove
    component props changed. The corrected focused gate and runtime gate both pass.
  - focused recipe command:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib switch_read_only_semantics_update_when_policy_model_changes -- --nocapture`
  - focused recipe result:
    passed, 1 test.
  - protocol roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_switch_read_only_dynamic_action_state script_v2_roundtrip_ui_gallery_switch_read_only_action_state --no-fail-fast`
  - protocol roundtrip result:
    passed, 2 tests; Nextest run id `be780bca-1c2b-4a26-9322-cdefd3de970a`.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-read-only-dynamic-action-state.json --dir .fret/diag/runs/ui-gallery-switch-read-only-dynamic-action-state-f113 --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778910608918`.
  - runtime evidence:
    `.fret/diag/runs/ui-gallery-switch-read-only-dynamic-action-state-f113/script.result.json`
  - packed evidence:
    `.fret/diag/runs/ui-gallery-switch-read-only-dynamic-action-state-f113/share/1778910608918.zip`
- Switch command-gated action-state runtime gate:
  `tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-command-gated-action-state.json`
  - invariant:
    external `WindowCommandEnabledService` command availability must update a non-list Switch's
    `disabled` and `invoke` semantics across frames, suppress checked-state mutation while disabled,
    and restore mutation after re-enabling without stale derived action-availability feedback.
  - implementation anchors:
    `ecosystem/fret-ui-kit/src/command.rs`,
    `ecosystem/fret-ui-shadcn/src/switch.rs`,
    `apps/fret-ui-gallery/src/driver/runtime_driver.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/switch/command_gate.rs`,
    `tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-command-gated-action-state.json`,
    and `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - finding:
    found a diagnostics harness gap. The Gallery driver handled the command after `UiTree`
    recorded a bubbling `handled=false` decision, but did not emit a driver-handled trace, so the
    strict runtime script could not prove command handling.
  - before runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-command-gated-action-state.json --dir .fret/diag/runs/ui-gallery-switch-command-gated-action-state-f114-final --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - before runtime result:
    failed at `wait_command_dispatch_trace_timeout`; best candidate was
    `ui_gallery.switch.command_gate.toggle_enabled` with `handled=false`.
  - protocol roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_switch_command_gated_action_state --no-fail-fast`
  - protocol roundtrip result:
    passed, 1 test; Nextest run id `ff37bc5e-7d16-492e-bf2e-cb1b0381993a`.
  - build gate:
    `cargo build --profile dev-fast -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-command-gated-action-state.json --dir .fret/diag/runs/ui-gallery-switch-command-gated-action-state-f114-final2 --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778914891818`.
  - runtime evidence:
    `.fret/diag/runs/ui-gallery-switch-command-gated-action-state-f114-final2/script.result.json`
  - packed evidence:
    `.fret/diag/runs/ui-gallery-switch-command-gated-action-state-f114-final2/share/1778914891818.zip`
- Shell theme/motion runtime token mutation gate:
  `tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-motion-preset-runtime-token-mutation.json`
  - invariant:
    Gallery shell Theme/Motion preset selections must update both the observable shell models and
    effective global Theme runtime tokens. Numeric motion-token assertions should use stable
    integer-scaled snapshot fields instead of strict equality on raw `f32` JSON values.
  - implementation anchors:
    `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
    `apps/fret-ui-gallery/src/driver/runtime_driver.rs`,
    `apps/fret-ui-gallery/src/driver/window_bootstrap.rs`,
    `tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-motion-preset-runtime-token-mutation.json`,
    `tools/diag-scripts/suites/ui-gallery-motion-pilot/suite.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - finding:
    found a diagnostics oracle gap. The first script draft could see the model/token transition, but
    strict JSON equality on raw `f32` easing/bounce values produced a false failure. The app
    snapshot now publishes rounded readable values plus milli-scaled integer fields for durable
    token gates.
  - protocol roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_motion_preset_runtime_token_mutation --no-fail-fast`
  - protocol roundtrip result:
    passed, 1 test; Nextest run id `8fc82fd2-4c72-49cb-883d-b6993fbaa4fd`.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-motion-preset-runtime-token-mutation.json --dir .fret/diag/runs/ui-gallery-motion-preset-runtime-token-mutation-f115c --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778919283142`.
  - runtime evidence:
    `.fret/diag/runs/ui-gallery-motion-preset-runtime-token-mutation-f115c/script.result.json`
  - packed evidence:
    `.fret/diag/runs/ui-gallery-motion-preset-runtime-token-mutation-f115c/share/1778919283142.zip`
- Platform preference runtime environment mutation gate:
  `tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-platform-preferences-runtime-environment-mutation.json`
  - invariant:
    Diagnostics-injected platform preferences must travel through the same runner-owned
    `WindowMetricsService` path as platform environment events, and both app snapshot readers and
    `ElementContext` environment queries must observe the resulting color scheme, reduced-motion,
    and text-scale values.
  - implementation anchors:
    `crates/fret-diag-protocol/src/lib.rs`,
    `crates/fret-runtime/src/effect.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`,
    `crates/fret-launch/src/runner/desktop/runner/effects.rs`,
    `crates/fret-launch/src/runner/web/effects.rs`,
    `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/motion_presets/environment_probe.rs`,
    `tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-platform-preferences-runtime-environment-mutation.json`, and
    `tools/diag-scripts/suites/ui-gallery-motion-pilot/suite.json`.
  - finding:
    found a harness script reliability defect. The first runtime run timed out while waiting for
    `ui-gallery-motion-presets-environment-probe` because the script never navigated from the
    default page into Motion Presets. The script now performs the same explicit nav-search and page
    click flow used by the existing Motion Presets scripts.
  - protocol roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_platform_preferences_runtime_environment_mutation script_v2_roundtrip_set_window_preferences_defaults --no-fail-fast`
  - protocol roundtrip result:
    passed, 2 tests; Nextest run id `f5713881-de6d-4eb3-9f4c-4d7d77e76697`.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed before the final script navigation hardening.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-platform-preferences-runtime-environment-mutation.json --dir .fret/diag/runs/ui-gallery-platform-preferences-runtime-environment-mutation-rerun --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - first runtime failure:
    `.fret/diag/runs/ui-gallery-platform-preferences-runtime-environment-mutation/script.result.json`
    reported `timeout.tooling.script_result`; the step sidecar
    `.fret/diag/runs/ui-gallery-platform-preferences-runtime-environment-mutation/1778922372693-script-step-0002-wait_until-timeout/test_ids.index.json`
    contained `ui-gallery-nav-motion-presets` but not the page or probe ids.
  - runtime result after script navigation hardening:
    passed; run id `1778922706072`.
  - runtime evidence:
    `.fret/diag/runs/ui-gallery-platform-preferences-runtime-environment-mutation-rerun/script.result.json`
  - packed evidence:
    `.fret/diag/runs/ui-gallery-platform-preferences-runtime-environment-mutation-rerun/share/1778922706072.zip`
- Diagnostics authoring page-entry lint gate:
  `tools/check_diag_scripts_registry.py`
  - scope:
    promoted `ui-gallery-motion-pilot` scripts, Motion Presets page-local selectors with
    `ui-gallery-motion-presets-*` test ids.
  - invariant:
    a script may not wait for, click, scroll, move to, or capture a page-local Motion Presets
    selector until it has first proved the owning page root `ui-gallery-page-motion-presets`.
    The always-visible shell motion preset trigger is allowlisted because it exists outside the
    Motion Presets page body.
  - finding:
    found and fixed an existing script authoring debt:
    `ui-gallery-motion-presets-fluid-tabs-pixels-changed-fixed-frame-delta.json` navigated to
    Motion Presets but waited for a page-local trigger without first asserting the page root.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`,
    `tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-motion-presets-fluid-tabs-pixels-changed-fixed-frame-delta.json`,
    `tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-platform-preferences-runtime-environment-mutation.json`, and
    `tools/diag-scripts/suites/ui-gallery-motion-pilot/suite.json`.
  - lint self-test gate:
    `python tools/test_check_diag_scripts_registry.py`
  - lint self-test result:
    passed, 3 tests.
  - registry gate:
    `python tools/check_diag_scripts_registry.py`
  - registry result:
    passed.
  - follow-up audit result:
    promoted Select scripts had 0 page-entry violations; promoted Combobox scripts initially had
    36 under a page-root-only rule, but they had explicit
    `FRET_UI_GALLERY_START_PAGE=combobox` defaults, so start-page defaults now count as valid entry
    evidence and Combobox is strict too.
  - DataTable follow-up:
    the first candidate audit reported 174 DataTable violations across 21 promoted scripts under a
    single-root model. The actual gap was diagnostics rule expressiveness: promoted DataTable
    scripts prove entry through several valid variant roots such as
    `ui-gallery-data-table-default-root`, `ui-gallery-data-table-basic-root`,
    `ui-gallery-data-table-listlike-root`, `ui-gallery-data-table-reusable-root`,
    `ui-gallery-data-table-rtl-root`, and `ui-gallery-data-table-torture-root`. The page-entry
    lint now supports `entry_ids`, strict DataTable page-entry is enabled for
    `ui-gallery-data-table`, `ui-gallery-data-table-retained`, and
    `ui-gallery-data-table-view-cache-torture`, and the candidate violation count is 0.
  - current lint self-test gate:
    `python tools/test_check_diag_scripts_registry.py`
  - current lint self-test result:
    passed, 11 tests.
  - current registry gate:
    `python tools/check_diag_scripts_registry.py`
  - current registry result:
    passed.
- AlertAction component-slot marker gate:
  - invariant:
    recipe-internal slot classification must not use globally exported diagnostics `test_id`s.
    Internal slots should be discoverable by composition code without entering semantics, layout,
    hit testing, or accessibility surfaces.
  - implementation anchors:
    `crates/fret-ui/src/element.rs` and `ecosystem/fret-ui-shadcn/src/alert.rs`.
  - focused gate:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib alert_action`
  - focused result:
    passed, 6 tests.
  - runtime gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-tabs-shared-indicator-pixels-changed-fixed-frame-delta.json --dir .fret/diag/runs/ui-gallery-alert-tabs-component-slot-fix --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1778926130298`.
  - packed evidence:
    `.fret/diag/runs/ui-gallery-alert-tabs-component-slot-fix/share/1778926130298.zip`
- Drawer snap-point visible-click and dismiss-contract gates:
  - invariant:
    long-page Drawer scripts must prove a trigger is visible/hittable before `click_stable`, and
    snap-point drag gates must encode the component's actual Vaul-style policy: releasing near a
    snap point settles, while dragging far enough down can dismiss and should restore focus.
  - implementation anchors:
    `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`, and
    `tools/diag-scripts/ui-gallery/drawer/`.
  - diagnostics unit gate:
    `cargo test --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics --lib click_stable_timeout`
  - diagnostics unit result:
    passed, 3 tests.
  - reason-code gate:
    `cargo test --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics --lib labels_reason_code_tests`
  - reason-code result:
    passed, 1 test.
  - runtime gates:
    `ui-gallery-drawer-snap-points-drag-retarget-settle-fixed-frame-delta.json`,
    `ui-gallery-drawer-snap-points-drag-settle.json`, and
    `ui-gallery-drawer-snap-points-spring-midflight-retarget-fixed-frame-delta.json`.
  - runtime results:
    passed with run ids `1778928529130`, `1778928579053`, and `1778928911018`.
  - packed evidence:
    `.fret/diag/runs/ui-gallery-drawer-snap-points-visible-click-fix/share/1778928529130.zip`,
    `.fret/diag/runs/ui-gallery-drawer-snap-points-drag-settle-visible-click-fix/share/1778928579053.zip`,
    and
    `.fret/diag/runs/ui-gallery-drawer-snap-points-spring-dismiss-contract-fix-2/share/1778928911018.zip`.
- Sidebar tooling-timeout evidence and long-page visibility gate:
  - invariant:
    when a script is stuck in a long-running intent step such as `click_stable`, external
    script-result timeout handling must leave a bounded bundle with selector, hit-test, and
    click-stable traces; long-page Sidebar content targets must be scrolled into view or
    bounds-checked before `click_stable`.
  - status:
    fixed. The confirmed issue was a diagnostics/tooling plus script-authoring gap, not a Sidebar
    component defect.
  - original repro:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-toggle-fixed-frame-delta.json --dir .fret/diag/runs/ui-gallery-sidebar-toggle-triage --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - original repro result:
    failed with `timeout.tooling.script_result`; run `1778930062035` remained at step 8
    `click_stable` and did not produce a forced bundle.
  - timeout-bundle fix anchors:
    `crates/fret-diag/src/tooling_failures.rs` and `crates/fret-diag/src/tests.rs`.
  - timeout-bundle regression gate:
    `cargo test --profile dev-fast -p fret-diag --lib run_script_over_transport_timeout_captures_last_bundle_when_run_started -- --nocapture`
  - timeout-bundle regression result:
    passed.
  - forced-bundle triage evidence:
    `.fret/diag/runs/ui-gallery-sidebar-toggle-triage3/script.result.json` reports
    `timeout.tooling.script_result` at step 8 and records `last_bundle_dir=1778934305640-diag-run`.
  - root-cause evidence:
    `.fret/diag/runs/ui-gallery-sidebar-toggle-triage3/sidebar-step8.slice.json` shows
    `ui-gallery-sidebar-demo-toggle` at `x=549.3333`, `y=1401.3333`, `w=28`, `h=28` in a
    `1080x720` window.
  - script/lint fix anchors:
    `tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-toggle-fixed-frame-delta.json`,
    `tools/check_diag_scripts_registry.py`, and `tools/test_check_diag_scripts_registry.py`.
  - registry lint gates:
    `python tools/test_check_diag_scripts_registry.py` and
    `python tools/check_diag_scripts_registry.py`
  - registry lint results:
    passed; the unit gate covers the Sidebar long-page negative case.
  - focused runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-toggle-fixed-frame-delta.json --dir .fret/diag/runs/ui-gallery-sidebar-toggle-triage4 --timeout-ms 45000 --poll-ms 20 --launch target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed, run id `1778936393221`.
  - current evidence:
    `.fret/diag/runs/ui-gallery-sidebar-toggle-triage/script.result.json` and
    `.fret/diag/runs/ui-gallery-sidebar-toggle-triage/1778930062035/script.result.json`,
    `.fret/diag/runs/ui-gallery-sidebar-toggle-triage3/script.result.json`,
    `.fret/diag/runs/ui-gallery-sidebar-toggle-triage3/sidebar-step8.slice.json`, and
    `.fret/diag/runs/ui-gallery-sidebar-toggle-triage4/script.result.json`.
- Shadcn structural slot hygiene:
  - invariant:
    recipe-internal child classification must use `AnyElement::component_slot`, not diagnostics
    `test_id` and not shortcut `key_context`.
  - finding:
    CardAction/CardFooter/AvatarBadge used generated diagnostics `test_id` markers; ItemMedia and
    ItemDescription used `key_context` for internal recipe classification.
  - implementation anchors:
    `ecosystem/fret-ui-shadcn/src/card.rs`,
    `ecosystem/fret-ui-shadcn/src/avatar.rs`, and
    `ecosystem/fret-ui-shadcn/src/item.rs`.
  - source-hygiene gate:
    `tools/check_shadcn_internal_slots.py` and `tools/test_check_shadcn_internal_slots.py`.
  - source audit:
    `rg -n "fret-ui-shadcn\\." ecosystem/fret-ui-shadcn/src -g "*.rs"` now reports only
    `component_slot` constants/usages in Alert, Avatar, Card, and Item.
  - negative source audit:
    `rg -n "key_context\\([^\\)]*fret-ui-shadcn|attach_test_id\\([^\\n]*fret-ui-shadcn|test_id\\([^\\n]*fret-ui-shadcn" ecosystem/fret-ui-shadcn/src -g "*.rs"`
    returns no matches.
  - focused gates:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib card_action_marker -- --nocapture`,
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib card_header_with_action_uses_explicit_grid_slot_placement -- --nocapture`,
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib card_sections_can_inherit_or_override_size -- --nocapture`,
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib avatar_badge_can_inherit_or_override_size -- --nocapture`,
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib item_structural_slots_do_not_use_key_context_or_diagnostics_test_id -- --nocapture`,
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib item_sized_provides_size_defaults_to_parts -- --nocapture`, and
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib item_media_with_description_self_starts_and_offsets_from_top -- --nocapture`.
  - focused gate results:
    all passed.
  - source-hygiene gate commands:
    `python tools/test_check_shadcn_internal_slots.py` and
    `python tools/check_shadcn_internal_slots.py`
  - source-hygiene gate results:
    passed, 4 tests and a clean source scan.
- Sonner named-toaster scoping:
  - invariant:
    an unnamed Toaster renders only unnamed toasts, while a named Toaster renders only toasts with
    the matching `toaster_id`; a single toast store entry must not appear in multiple live toast
    overlay stacks.
  - finding:
    `ui-gallery-motion-pilot` found duplicate `toast-entry-1` and `toast-entry-2` semantics
    `test_id`s after the Sonner interrupt gate because the shell Toaster rendered page-local
    named toasts in addition to the page-local named Toaster.
  - implementation anchors:
    `ecosystem/fret-ui-kit/src/window_overlays/render.rs` and
    `ecosystem/fret-ui-kit/src/window_overlays/tests/toast.rs`.
  - focused unit gate:
    `cargo test --profile dev-fast -p fret-ui-kit --lib toast_layers_scope_named_toasts_to_matching_toaster_id -- --nocapture`
  - focused unit result:
    passed.
  - focused runtime gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/sonner/ui-gallery-sonner-interrupt-fixed-frame-delta.json --dir .fret/diag/runs/ui-gallery-sonner-interrupt-after-scope-fix --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed, run id `1778939842586`, share pack
    `.fret/diag/runs/ui-gallery-sonner-interrupt-after-scope-fix/share/1778939842586.zip`.
  - duplicate audit:
    `target/dev-fast/fretboard-dev.exe diag test-ids .fret/diag/runs/ui-gallery-sonner-interrupt-after-scope-fix --json --max-test-ids 20`
  - duplicate audit result:
    `duplicate_test_ids_total=0`; `toast-entry-1` and `toast-entry-2` each have `count=1`.
  - lint evidence:
    `.fret/diag/runs/ui-gallery-sonner-interrupt-after-scope-fix/1778939851780-ui-gallery-sonner-interrupt-fixed-frame-delta/check.lint.json`
    reports `error_issues=0`.
  - full-suite follow-up:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-motion-pilot --dir .fret/diag/runs/ui-gallery-motion-pilot-after-toast-scope-fix --timeout-ms 900000 --session-auto --launch target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    passed, 14/14 rows; summary
    `.fret/diag/runs/ui-gallery-motion-pilot-after-toast-scope-fix/sessions/1778940056096-94540/suite.summary.json`.
- Sonner toast action/cancel accessible names:
  - invariant:
    toast action and cancel controls are interactive buttons, so the visual `ToastAction.label`
    must also be exported as the button accessible name.
  - finding:
    after the named-toaster scoping fix, `ui-gallery-sonner-interrupt-fixed-frame-delta.json`
    still produced one `semantics.missing_label` warning. The flagged button was the visible
    `Undo` toast action: the child text node existed, but the button itself had no label/value.
  - implementation anchors:
    `ecosystem/fret-ui-kit/src/window_overlays/render.rs` and
    `ecosystem/fret-ui-kit/src/window_overlays/tests/toast.rs`.
  - focused unit gate:
    `cargo test --profile dev-fast -p fret-ui-kit --lib toast_action_and_cancel_labels_are_exposed_in_semantics_snapshot -- --nocapture`
  - focused unit result:
    passed.
  - focused runtime gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/sonner/ui-gallery-sonner-interrupt-fixed-frame-delta.json --dir .fret/diag/runs/ui-gallery-sonner-interrupt-after-action-label-fix --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed, run id `1778941669635`, share pack
    `.fret/diag/runs/ui-gallery-sonner-interrupt-after-action-label-fix/share/1778941669635.zip`.
  - focused lint:
    `target/dev-fast/fretboard-dev.exe diag lint .fret/diag/runs/ui-gallery-sonner-interrupt-after-action-label-fix/1778941679023-ui-gallery-sonner-interrupt-fixed-frame-delta/bundle.schema2.json --json --out .fret/diag/runs/ui-gallery-sonner-interrupt-after-action-label-fix/1778941679023-ui-gallery-sonner-interrupt-fixed-frame-delta/check.lint.json`
  - focused lint result:
    `error_issues=0`, `warning_issues=0`.
  - full-suite follow-up:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-motion-pilot --dir .fret/diag/runs/ui-gallery-motion-pilot-after-toast-action-label-fix --timeout-ms 900000 --session-auto --launch target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    passed, 14/14 rows; summary
    `.fret/diag/runs/ui-gallery-motion-pilot-after-toast-action-label-fix/sessions/1778941744107-108312/suite.summary.json`.
- Carousel demo inner-button accessible name:
  - invariant:
    UI Gallery fixture/demo controls that are exposed as interactive semantics nodes must have a
    stable accessible name, even when their visual label is intentionally empty.
  - finding:
    three Carousel motion-pilot scripts reported `semantics.missing_label` for
    `ui-gallery-carousel-demo-inner-button`; the demo used `Button::new("")` without
    `.a11y_label(...)`.
  - implementation anchor:
    `apps/fret-ui-gallery/src/ui/snippets/carousel/demo.rs`.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - focused runtime gates:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery-carousel-expandable-fixed-frame-delta.json --dir .fret/diag/runs/ui-gallery-carousel-expandable-after-inner-label-fix --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery-carousel-focus-watch-tab-scrolls-gate.json --dir .fret/diag/runs/ui-gallery-carousel-focus-watch-after-inner-label-fix --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery-carousel-loop-continuity-touch-gate.json --dir .fret/diag/runs/ui-gallery-carousel-loop-continuity-after-inner-label-fix --timeout-ms 300000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - focused runtime results:
    passed with run ids `1778942849445`, `1778943027333`, and `1778943151863`.
  - focused lint evidence:
    `.fret/diag/runs/ui-gallery-carousel-expandable-after-inner-label-fix/1778942966450-ui-gallery-carousel-expandable-fixed-frame-delta/check.lint.json`,
    `.fret/diag/runs/ui-gallery-carousel-focus-watch-after-inner-label-fix/1778943117290-ui-gallery-carousel-focus-watch-tab-scrolls/check.lint.json`, and
    `.fret/diag/runs/ui-gallery-carousel-loop-continuity-after-inner-label-fix/1778943271162-ui-gallery-carousel-loop-continuity-end/check.lint.json`.
  - focused lint results:
    all report `error_issues=0`, `warning_issues=0`.
- Tabs shared-indicator non-empty diagnostics bounds:
  - invariant:
    decorative visual surfaces that carry generated diagnostics `test_id`s must have non-empty
    bounds even when pointer hit-testing is disabled.
  - finding:
    the Motion Presets `ui-gallery-motion-presets-fluid-tabs-shared-indicator` node had zero
    semantics bounds because the Tabs shared indicator used a default auto-sized
    `hit_test_gate(false)` around an absolute canvas. Fret's self-drawn layout path needs explicit
    Fill sizing here; CSS-style inset fill is not enough.
  - implementation anchor:
    `ecosystem/fret-ui-shadcn/src/tabs.rs`.
  - focused unit gate:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib tabs_shared_indicator_test_id_has_non_empty_bounds -- --nocapture`
  - focused unit result:
    passed.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - focused runtime gates:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-platform-preferences-runtime-environment-mutation.json --dir .fret/diag/runs/ui-gallery-platform-preferences-after-tabs-indicator-fill --timeout-ms 240000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-motion-presets-fluid-tabs-pixels-changed-fixed-frame-delta.json --dir .fret/diag/runs/ui-gallery-fluid-tabs-after-tabs-indicator-fill --timeout-ms 300000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - focused runtime results:
    passed with run ids `1778944327922` and `1778944371703`; focused lint reports
    `error_issues=0`, `warning_issues=0` for both bundles.
  - full-suite follow-up:
    `.fret/diag/runs/ui-gallery-motion-pilot-after-carousel-tabs-cleanup/sessions/1778944420308-93620/suite.summary.json`
  - full-suite result:
    passed, 14/14 rows, `scripts_with_evidence=14`, `focus_mismatch_total=0`,
    `lint_error_total=0`, `lint_warning_total=0`.
- Motion-pilot zero-warning suite policy:
  - invariant:
    a clean motion-pilot suite must mean the runtime scripts pass and each script's diagnostics
    lint has `error_issues=0` and `warning_issues=0`.
  - implementation anchors:
    `crates/fret-diag/src/diag_suite.rs`,
    `tools/diag-scripts/suites/ui-gallery-motion-pilot/suite.json`, and
    `tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-toggle-fixed-frame-delta.json`.
  - focused policy gates:
    `cargo test --profile dev-fast -p fret-diag --lib suite_lint_policy -- --nocapture`,
    `cargo test --profile dev-fast -p fret-diag --lib lint_warning_budget -- --nocapture`,
    `cargo test --profile dev-fast -p fret-diag --lib maybe_run_suite_script_lint -- --nocapture`,
    and
    `cargo test --profile dev-fast -p fret-diag --lib finalize_suite_script_success_tail_records_row_when_lint_and_post_run_skip -- --nocapture`.
  - focused Sidebar runtime gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-toggle-fixed-frame-delta.json --dir .fret/diag/runs/ui-gallery-sidebar-toggle-fixed-frame-delta-stable-entry-v1 --timeout-ms 300000 --pack --include-triage --include-screenshots --launch target/dev-fast/fret-ui-gallery.exe`
  - focused Sidebar runtime/lint result:
    passed with run id `1778949248838`; focused lint reports `error_issues=0`,
    `warning_issues=0`.
  - full-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-motion-pilot --dir .fret/diag/runs/ui-gallery-motion-pilot-lint-policy-v2 --timeout-ms 1200000 --session-auto --launch target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    `.fret/diag/runs/ui-gallery-motion-pilot-lint-policy-v2/sessions/1778949363628-19720/suite.summary.json`
    reports `status=passed`, 14/14 rows, `scripts_with_evidence=14`,
    `focus_mismatch_total=0`, `lint_error_total=0`, `lint_warning_total=0`, and
    `failed_policy=0`.
- ScrollArea strict click-visibility and capture-state diagnostics:
  - invariant:
    promoted long-page content clicks must prove target visibility before pointer synthesis, and
    current-state debug predicates must read the latest debug snapshot rather than a historical ring
    aggregate. Promoted pointer/capture scripts must also wait for bounded current-state
    convergence after pointer events instead of asserting the next step against a possibly stale
    debug snapshot.
  - findings:
    ScrollArea had five promoted long-page content clicks that still used plain `click`; enabling
    strict lint then exposed a diagnostics harness defect where `input_pointer_capture_active_is`
    could match stale debug-snapshot ring entries. The multi-pointer ScrollArea script also used
    immediate `assert` steps for cross-frame capture state and now waits for state convergence.
    A follow-up promoted-registry audit found three remaining event-adjacent current-state
    `assert` steps in the baseline and pointer-cancel ScrollArea scrollbar scripts; those scripts
    now use bounded `wait_until` steps and the registry lint rejects future promoted regressions.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_predicates.rs`,
    `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json`,
    `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-pointer-cancel-release.json`,
    and `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-multipointer-underlay-touch.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`,
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 15 tests. Structured promoted-registry audit reports
    `immediate assert violations=0` and `adjacent wait_until convergence patterns=6`.
  - focused predicate gate:
    `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics current_state_predicates_do_not_match_stale_ring_snapshots input_pointer_capture_active_predicate_reads_debug_snapshot --no-fail-fast`
  - focused predicate result:
    passed with Nextest run id `70f41d32-7ce9-4d1d-9ea9-680d61f909d3`.
  - focused runtime gate:
    `target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-multipointer-underlay-touch.json --dir target/fret-diag-scrollbar-drag-multipointer-underlay-touch-after-wait-until-v1 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target/debug/fret-ui-gallery.exe`
  - focused runtime result:
    passed with run id `1778955752839`.
  - follow-up focused runtime gates:
    `target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json --dir target/fret-diag-scrollbar-drag-baseline-content-growth-wait-until-v1 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target/debug/fret-ui-gallery.exe`
    `target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-pointer-cancel-release.json --dir target/fret-diag-scrollbar-drag-pointer-cancel-release-wait-until-v1 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target/debug/fret-ui-gallery.exe`
  - follow-up focused runtime results:
    passed with run ids `1778956466823` and `1778956486039`.
  - full-suite gate:
    `target/debug/fretboard-dev.exe diag suite ui-gallery-scroll-area --dir target/fret-diag-scroll-area-suite-pointer-current-state-lint-v1 --session-auto --timeout-ms 360000 --launch -- target/debug/fret-ui-gallery.exe`
  - full-suite result:
    `target/fret-diag-scroll-area-suite-pointer-current-state-lint-v1/sessions/1778956501773-46724/suite.summary.json`
    reports the suite passed.
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
