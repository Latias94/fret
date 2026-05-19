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
column wrap width, max-width row wrap width, and overflow/scale constraints. It also locks flex
visual-order consistency: `FlexItemStyle.order` is now applied through the same ordered child
sequence in layout and intrinsic measurement, including wrap-sensitive measurement cases
(`b269764aa2`).

The same fixture now also covers auto-sized container child margin accounting: max-content
measurement for a margin-bearing child now matches the laid-out container bounds, so measurement
and layout stay aligned for finite margins in the auto-container path.

The layout primitive fixture now includes `render-transform-mixed-flow-absolute-envelope-matches-visual-hit`.
It reuses the mixed Pressable flow/absolute envelope case under a `RenderTransform` and proves the
`34 x 12` layout/measurement envelope is preserved while visual and hit spaces translate together.
The layout-space near-edge sample misses the absolute child, while the translated visual-space
near-edge sample hits the absolute child. No new mechanism fix was required.

The same fixture now also includes
`fractional-render-transform-derives-visual-hit-from-layout-size`. It proves
`FractionalRenderTransform` computes a size-derived `40 x 10` translation from a `20 x 20`
interactive child while keeping layout bounds authoritative and moving visual/hit spaces together.
The layout-space center misses the target, and the translated visual-space center hits it. No new
mechanism fix was required.

Evidence anchor:

- `crates/fret-ui/src/declarative/host_widget/measure.rs`
- `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`
- `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`

Run result:

- `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  - Result: passed; Nextest run id `8a831141-fd89-4656-be5b-59a3d206bdef`.
- `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\layout_primitives_v1.json`
  - Result: passed.

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
- Moving mixed flow/absolute wrapper focused gate:
  `crates/fret-ui/src/declarative/tests/view_cache.rs`
  - proof:
    `view_cache_hit_moving_mixed_absolute_wrapper_updates_bounds_and_hit_test` keeps the ViewCache
    child render closure clean while a parent spacer moves a mixed flow/absolute `Pressable`
    subtree. It asserts moved layout bounds, moved element visual bounds, moved absolute-child
    bounds, fallback hit-testing, and runtime routing via `debug_hit_test_routing`.
  - first red command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_hit_moving_mixed_absolute_wrapper_updates_bounds_and_hit_test --no-fail-fast --no-capture`
  - first red result:
    failed with Nextest run id `ff29d5c0-ec02-45d7-bc41-63956a5020fb`; the harness expected the
    old point to hit nothing, but it legitimately hit the expanded outer row after spacer
    insertion. The stale absolute-child hit condition did not reproduce.
  - current focused command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_hit_moving_mixed_absolute_wrapper_updates_bounds_and_hit_test --no-fail-fast --no-capture`
  - current focused result:
    passed with Nextest run id `7e88f25a-7ea4-42c2-96d6-781f9da9482d`.
  - family command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache --no-fail-fast`
  - family result:
    passed, 66/66 tests, with Nextest run id `a49ebb69-7e9a-4b66-bc83-7f5f52d35a0e`.
  - formatting:
    `cargo fmt -p fret-ui --check` passed.

## Timer Dispatch Lifecycle Gates

```powershell
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_timer_dispatch_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib timer_dispatch -- --nocapture
cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev
target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/select/ui-gallery-select-typeahead-commit-banana.json --dir target/fret-diag-select-typeahead-input-snapshot-after-hidden-timer-target-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe
target/dev-fast/fretboard-dev.exe diag suite ui-gallery-select --dir target/fret-diag-select-suite-after-hidden-timer-target-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe
```

Current evidence anchors:

- Timer lifecycle fixture:
  `crates/fret-ui/src/tree/tests/fixtures/timer_dispatch_v1.json`
  - runner:
    `crates/fret-ui/src/tree/tests/timer_dispatch_harness.rs`
  - proof:
    covers visible base targets, visible hit-test-inert transition overlay targets, hidden overlay
    targets, and removed overlay targets.
  - current command:
    `cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_timer_dispatch_matches_oracles -- --nocapture`
  - current result:
    passed.
- Focused timer family gate:
  `cargo test --profile dev-fast -p fret-ui --lib timer_dispatch -- --nocapture`
  - current result:
    passed, 7 tests.
- Runtime Select typeahead regression:
  `target/fret-diag-select-typeahead-input-snapshot-after-hidden-timer-target-v2/sessions/1778963942817-47376/1778963951450/ai.packet`
  - current result:
    passed; `tooling_warnings=[]`; no `dispatch/chain`, `node missing from input snapshot`,
    `layout.zero_size`, or underflow text found in the packet.
- Runtime Select suite follow-up:
  `target/fret-diag-select-suite-after-hidden-timer-target-v1/sessions/1778964075071-40956/suite.summary.json`
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
- Retained Table runtime suite:
  `tools/diag-scripts/suites/ui-gallery-table-retained/suite.json`
  - proof:
    runs the retained Table torture surface across keyboard typeahead, multi-sort, row pinning with
    `keep_pinned_rows` true and false, descending sort, sort/select/scroll, and a retained
    window-boundary scroll bounce that asserts both row-window movement and aggregate retained
    reconcile telemetry.
  - focused evidence:
    `target/fret-diag-table-retained-window-boundary-scroll-focused-v4/sessions/1779037981653-160576/1779038066334/ai.packet`
  - suite evidence:
    `target/fret-diag-table-retained-suite-candidate-v4/sessions/1779038155054-118992/suite.summary.json`
  - suite result:
    passed, 7/7 rows, `scripts_with_evidence=6`, `focus_mismatch_total=0`, zero lint
    errors/warnings.
  - companion harness gates:
    `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics history_predicates_can_match_stale_latest_snapshot retained_virtual_list_reconciles_matching_predicate_counts_ring_snapshots no_frame_keepalive_does_not_consume_wait_frames --no-fail-fast`
    passed with latest Nextest run id `3e4a180a-2670-4a6d-a826-ec84d196fdec`.
- Retained Table selected semantics focused gate:
  `ecosystem/fret-ui-kit/src/declarative/table.rs`
  - proof:
    `table_virtualized_retained_selected_semantics_follow_windowed_row_selection` renders a
    retained Table, asserts row 0 starts with `SemanticsNode.flags.selected=false`, clicks row 0,
    asserts it refreshes to `true`, then scrolls the retained window to row 25 and proves row 25 is
    unselected while row 0 is absent from the current semantics snapshot.
  - focused command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_selected_semantics_follow_windowed_row_selection --no-fail-fast --no-capture`
  - focused result:
    passed; Nextest run id `bfefef11-f3dc-435a-a986-1d0cc16666d2`.
  - runtime companion:
    `tools/diag-scripts/ui-gallery/table/ui-gallery-table-retained-sort-select-scroll.json` now
    asserts row 0 `selected=false`, row 0 `selected=true` after click, row 25 `selected=false`
    after scrolling, and row 0 `not_exists`.
  - protocol gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_table_retained_sort_select_scroll script_v2_roundtrip_ui_gallery_table_retained_window_boundary_scroll --no-fail-fast`
    passed with Nextest run id `c6dd9233-dda9-48fd-8fde-c510fc6d9ac1`.
  - registry:
    `python tools\check_diag_scripts_registry.py`
    passed.
  - focused runtime rerun:
    `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\table\ui-gallery-table-retained-sort-select-scroll.json --dir target\fret-diag-table-retained-selected-sort-select-scroll-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
    failed before selected-state assertions at step 2,
    `bounds_within_window(ui-gallery-table-retained-header-row)`.
  - timeout evidence:
    `target/fret-diag-table-retained-selected-sort-select-scroll-v2/sessions/1779101071184-190392/script.result.json`;
    forced bundle
    `target/fret-diag-table-retained-selected-sort-select-scroll-v2/sessions/1779101071184-190392/1779101173854`;
    share pack
    `target/fret-diag-table-retained-selected-sort-select-scroll-v2/sessions/1779101071184-190392/share/1779101173854.zip`.
  - triage:
    the header-row selector matched one node, but its bounds were still `0,0 0x0`; this keeps the
    runtime selected assertions as authored-but-pending evidence rather than a failed retained
    Table selected-semantics proof.
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

- MaskLayer paint-only hit-testing fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/hit_test_routing_v1.json`
  - proof:
    `mask-layer-bounds-do-not-clip-hit-testing-by-default` proves an offset escaped child remains
    targetable outside `MaskLayer` bounds when the wrapper uses default visible overflow.
    `mask-layer-overflow-clip-suppresses-escaped-child-hit` proves the same escaped child is
    suppressed when the wrapper opts into `Overflow::Clip`.
  - first red command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  - first red result:
    failed with Nextest run id `54d2868e-b176-42cb-a6e2-ad01ce63b497`; the initial oracle used a
    width-overflow child that the layout contract legitimately constrained to wrapper width.
  - current command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  - current result:
    passed with Nextest run id `a72e3112-544e-405a-957d-d4d00dfad034`.
  - JSON validation:
    `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\hit_test_routing_v1.json`
    passed.
- EffectLayer computation-bound hit-testing fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/hit_test_routing_v1.json`
  - proof:
    `effect-layer-bounds-do-not-clip-hit-testing-by-default` proves an offset escaped child
    remains targetable outside `EffectLayer` bounds when the wrapper uses default visible
    overflow. `effect-layer-overflow-clip-suppresses-escaped-child-hit` proves the same escaped
    child is suppressed when the wrapper opts into `Overflow::Clip`.
  - current command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  - current result:
    passed with Nextest run id `c31f8473-555a-4b65-996c-0648d8f85b75`.
  - JSON validation:
    `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\hit_test_routing_v1.json`
    passed.
- CompositeGroup computation-bound hit-testing fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/hit_test_routing_v1.json`
  - proof:
    `composite-group-bounds-do-not-clip-hit-testing-by-default` proves an offset escaped child
    remains targetable outside `CompositeGroup` bounds when the wrapper uses default visible
    overflow. `composite-group-overflow-clip-suppresses-escaped-child-hit` proves the same escaped
    child is suppressed when the wrapper opts into `Overflow::Clip`.
  - current command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  - current result:
    passed with Nextest run id `eb30efc1-a5c9-40a1-84ae-8360753f0842`.
  - JSON validation:
    `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\hit_test_routing_v1.json`
    passed.
- BackdropSourceGroup computation-bound hit-testing fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/hit_test_routing_v1.json`
  - proof:
    `backdrop-source-group-bounds-do-not-clip-hit-testing-by-default` proves an offset escaped
    child remains targetable outside `BackdropSourceGroup` bounds when the wrapper uses default
    visible overflow. `backdrop-source-group-overflow-clip-suppresses-escaped-child-hit` proves the
    same escaped child is suppressed when the wrapper opts into `Overflow::Clip`.
  - current command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_hit_test_routing_matches_oracles --no-fail-fast --no-capture`
  - current result:
    passed with Nextest run id `05c28589-9f50-4aa2-b1d5-3ab3a3700de2`.
  - JSON validation:
    `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\hit_test_routing_v1.json`
    passed.
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
- Combobox long-text trigger/option geometry and renderer font-trace gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-long-text-geometry.json`
  - asserts trigger label width budget, label-before-chevron right delta, chrome-relative vertical
    centering, popup placement, option label width budget, option label/checkmark insets, and
    option label vertical centering.
  - now also enables `FRET_TEXT_FONT_TRACE_ALL=1` and asserts at least one renderer font trace
    entry for the selected "Enterprise Observability" label with `font=ui`, `wrap=none`,
    `overflow=ellipsis`, and `missing_glyphs=0`.
  - focused test:
    `combobox_trigger_long_label_stays_before_chevron`
  - evidence:
    `target/fret-diag-combobox-long-text-geometry-v4/sessions/1778619498565-104108/script.result.json`
  - bundle with long-text child anchors:
    `target/fret-diag-combobox-long-text-geometry-v4/sessions/1778619498565-104108/1778619501160-ui-gallery-combobox-long-text-open/bundle.schema2.json`
  - renderer font-trace predicate protocol/evaluator gates:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_render_text_font_trace_entries_matching_ge_serializes_and_deserializes script_v2_roundtrip_ui_gallery_combobox_long_text_geometry --no-fail-fast --no-capture`
    passed with Nextest run id `88d1c4cf-a5e7-4b17-91bd-998df1857420`.
    `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics render_text_font_trace_matching_predicate_matches_renderer_text_facts --no-fail-fast --no-capture`
    passed with Nextest run id `4ec491be-913d-47fa-b1e3-d7e756594342`.
  - renderer font-trace runtime evidence:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-long-text-geometry.json --dir target/fret-diag-combobox-long-text-geometry-renderer-trace-v4 --session-auto --pack --ai-packet --launch -- target/dev-fast/fret-ui-gallery.exe`
    passed with run id `1779077880731`.
  - renderer font-trace runtime artifacts:
    `target/fret-diag-combobox-long-text-geometry-renderer-trace-v4/sessions/1779077868548-173912/script.result.json`,
    `target/fret-diag-combobox-long-text-geometry-renderer-trace-v4/sessions/1779077868548-173912/1779077880731/ai.packet`,
    `target/fret-diag-combobox-long-text-geometry-renderer-trace-v4/sessions/1779077868548-173912/share/1779077880731.zip`
- Combobox RTL long-text trigger/option geometry and renderer font-trace gate:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-geometry.json`
  - asserts trigger label width budget, physical-left RTL chevron inset, label-after-chevron
    spacing, chrome-relative vertical centering, content-shell top collision flip with
    `side_offset_px=6`, option label width budget, physical-right RTL checkmark inset, and
    label-before-checkmark spacing.
  - now also enables `FRET_TEXT_FONT_TRACE_ALL=1` and asserts at least one renderer font trace
    entry for the selected "Enterprise Observability" label with `font=ui`, `wrap=none`,
    `overflow=ellipsis`, and `missing_glyphs=0`.
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
  - renderer font-trace script roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_geometry --no-fail-fast --no-capture`
    passed with Nextest run id `23514d59-c3bc-4985-8c8f-d1047d32e6aa`.
  - renderer font-trace runtime evidence:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-geometry.json --dir target/fret-diag-combobox-rtl-long-text-renderer-trace-v1 --session-auto --pack --ai-packet --include-screenshots --launch -- target/dev-fast/fret-ui-gallery.exe`
    passed with run id `1779078285665`.
  - renderer font-trace runtime artifacts:
    `target/fret-diag-combobox-rtl-long-text-renderer-trace-v1/sessions/1779078273314-137596/script.result.json`,
    `target/fret-diag-combobox-rtl-long-text-renderer-trace-v1/sessions/1779078273314-137596/1779078285665/ai.packet`,
    `target/fret-diag-combobox-rtl-long-text-renderer-trace-v1/sessions/1779078273314-137596/share/1779078285665.zip`
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
  - now also enables `FRET_TEXT_FONT_TRACE_ALL=1` and asserts a renderer font trace entry for the
    long grouped-input value with `font=ui`, `wrap=none`, `overflow=clip`, and
    `missing_glyphs=0`.
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
  - first renderer-trace runtime draft:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-long-text.json --dir target/fret-diag-button-group-input-group-long-text-renderer-trace-v1 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
    failed at step 7 with `wait_until_timeout` on
    `bounds_within_window(ui-gallery-button-group-input-group-control)`.
  - first-draft triage:
    `target/fret-diag-button-group-input-group-long-text-renderer-trace-v1/sessions/1779082623510-152148/1779082638377/ai.packet/slice.failed_step.7.test_id.ui-gallery-button-group-input-group-control.json`
    showed a unique `text_field` match for `ui-gallery-button-group-input-group-control` with
    pre-mutation bounds `0 x 0`, so the failure was a script precondition issue. The final script
    waits on the owning Button Group root before mutating the direct control and keeps the
    post-mutation control-size assertions.
  - renderer font-trace focused roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_button_group_input_group_long_text --no-fail-fast --no-capture`
    passed with Nextest run id `d656b55e-a80c-4a53-8cb0-98a8c2307872`.
  - renderer font-trace full roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip --no-fail-fast`
    passed with Nextest run id `72429701-d2a0-4d9f-9742-563fc421a36f`.
  - registry gates:
    `python tools/check_diag_scripts_registry.py` and
    `python tools/test_check_diag_scripts_registry.py` passed after refreshing
    `tools/diag-scripts/index.json`.
  - renderer font-trace runtime evidence:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-input-group-long-text.json --dir target/fret-diag-button-group-input-group-long-text-renderer-trace-v2 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
    passed with run id `1779082851147`.
  - renderer font-trace runtime artifacts:
    `target/fret-diag-button-group-input-group-long-text-renderer-trace-v2/sessions/1779082839430-157148/script.result.json`,
    `target/fret-diag-button-group-input-group-long-text-renderer-trace-v2/sessions/1779082839430-157148/1779082851147/ai.packet`,
    `target/fret-diag-button-group-input-group-long-text-renderer-trace-v2/sessions/1779082839430-157148/share/1779082851147.zip`,
    `target/fret-diag-button-group-input-group-long-text-renderer-trace-v2/sessions/1779082839430-157148/1779082919330-ui-gallery-button-group-input-group-long-text.layout/layout.taffy.v1.json`, and
    `target/fret-diag-button-group-input-group-long-text-renderer-trace-v2/sessions/1779082839430-157148/screenshots/1779082919566-ui-gallery-button-group-input-group-long-text/window-4294967297-tick-6-frame-6.png`.
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
  - now also enables `FRET_TEXT_FONT_TRACE_ALL=1` and asserts at least one renderer font trace
    entry for the long query with `font=ui`, `wrap=none`, `overflow=clip`, and
    `missing_glyphs=0`.
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
  - renderer font-trace focused roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_command_docs_demo_long_query_text --no-fail-fast --no-capture`
    passed with Nextest run id `347bc280-0e4c-4f1d-beed-062fd2e4903f`.
  - renderer font-trace full roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip --no-fail-fast`
    passed with Nextest run id `9f94f99d-86c7-417b-9b0f-5f29e4ca5797`.
  - renderer font-trace runtime evidence:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-docs-demo-long-query-text.json --dir target/fret-diag-command-docs-demo-long-query-renderer-trace-v6 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
    passed with run id `1779080299508`.
  - renderer font-trace runtime artifacts:
    `target/fret-diag-command-docs-demo-long-query-renderer-trace-v6/sessions/1779080286919-178616/script.result.json`,
    `target/fret-diag-command-docs-demo-long-query-renderer-trace-v6/sessions/1779080286919-178616/1779080299508/ai.packet`,
    `target/fret-diag-command-docs-demo-long-query-renderer-trace-v6/sessions/1779080286919-178616/share/1779080299508.zip`,
    `target/fret-diag-command-docs-demo-long-query-renderer-trace-v6/sessions/1779080286919-178616/1779080386435-ui-gallery-command-docs-demo-long-query-text.layout/layout.taffy.v1.json`, and
    `target/fret-diag-command-docs-demo-long-query-renderer-trace-v6/sessions/1779080286919-178616/screenshots/1779080386835-ui-gallery-command-docs-demo-long-query-text/window-4294967297-tick-7-frame-7.png`.
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
  - now also starts directly on the Input page with `FRET_UI_GALLERY_START_PAGE=input`, enables
    `FRET_TEXT_FONT_TRACE_ALL=1`, and asserts renderer font trace entries for both long values
    with `font=ui`, `wrap=none`, `overflow=clip`, and `missing_glyphs=0`.
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
  - renderer font-trace focused roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_input_basic_and_file_long_text --no-fail-fast --no-capture`
    passed with Nextest run id `4fc33be1-73de-4abd-b44e-1372c97cbe10`.
  - renderer font-trace full roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip --no-fail-fast`
    passed with Nextest run id `29fe790a-4b52-4b96-9d3c-0fb7677a8401`.
  - registry gates:
    `python tools/check_diag_scripts_registry.py` and
    `python tools/test_check_diag_scripts_registry.py` passed after refreshing
    `tools/diag-scripts/index.json`.
  - renderer font-trace runtime evidence:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/input/ui-gallery-input-basic-and-file-long-text.json --dir target/fret-diag-input-basic-file-long-text-renderer-trace-v1 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
    passed with run id `1779081108865`.
  - renderer font-trace runtime artifacts:
    `target/fret-diag-input-basic-file-long-text-renderer-trace-v1/sessions/1779081088220-65512/script.result.json`,
    `target/fret-diag-input-basic-file-long-text-renderer-trace-v1/sessions/1779081088220-65512/1779081108865/ai.packet`,
    `target/fret-diag-input-basic-file-long-text-renderer-trace-v1/sessions/1779081088220-65512/share/1779081108865.zip`,
    `target/fret-diag-input-basic-file-long-text-renderer-trace-v1/sessions/1779081088220-65512/1779081309030-ui-gallery-input-basic-and-file-long-text.layout/layout.taffy.v1.json`, and
    `target/fret-diag-input-basic-file-long-text-renderer-trace-v1/sessions/1779081088220-65512/screenshots/1779081309581-ui-gallery-input-basic-and-file-long-text/window-4294967297-tick-23-frame-23.png`.
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
  - ownership-close suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-resizable --dir target/fret-diag-resizable-suite-ownership-close-v1 --session-auto --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - ownership-close suite result:
    `target/fret-diag-resizable-suite-ownership-close-v1/sessions/1778976095291-118960/suite.summary.json`
    reports `status=passed`, 1/1 row, and zero lint errors/warnings.
- Moving cached Combobox view-cache/root-boundary gate:
  `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json`
  - invariant:
    a cached Combobox source that moves between Resizable panels must keep paint, semantics,
    hit-test routing, and overlay placement in the same coordinate space even when the source
    subtree is reused through ViewCache rather than rerendered.
  - implementation anchors:
    `crates/fret-ui/src/tree/prepaint/mod.rs`,
    `crates/fret-ui/src/tree/prepaint/interaction.rs`,
    `crates/fret-ui/src/tree/tests/prepaint.rs`,
    `apps/fret-ui-gallery/src/ui/snippets/resizable/moving_cached_combobox.rs`,
    `apps/fret-ui-gallery/src/ui/pages/resizable.rs`,
    `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json`,
    and `tools/diag-scripts/suites/ui-gallery-resizable/suite.json`.
  - failed evidence before fix:
    `target/fret-diag-resizable-moving-cached-combobox-v5/sessions/1779027102545-105712/script.result.json`
  - failed result:
    step 17 timed out waiting for `ui-gallery-resizable-view-cache-moving-combobox-input`; the
    click trace had trigger semantics bounds in the right panel but hit routing resolved to the
    right panel container because replayed interaction records retained their old absolute bounds.
  - focused prepaint regression:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui prepaint_interaction_cache_replay_translates_records_when_cache_root_moves --no-capture`
  - focused prepaint result:
    passed, 1 test.
  - prepaint family filter:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui prepaint --no-capture`
  - prepaint family result:
    passed, 20 tests.
  - hit-test/view-cache transform guard:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui hit_test_works_with_view_cache_root_and_prepaint_reuse_under_render_transform --no-capture`
  - hit-test/view-cache transform result:
    passed, 1 test.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json --dir target/fret-diag-resizable-moving-cached-combobox-v6 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime result:
    passed; run id `1779027983606`.
  - runtime evidence:
    `target/fret-diag-resizable-moving-cached-combobox-v6/sessions/1779027970415-65904/script.result.json`
  - AI packet:
    `target/fret-diag-resizable-moving-cached-combobox-v6/sessions/1779027970415-65904/1779027983606/ai.packet`
  - packed evidence:
    `target/fret-diag-resizable-moving-cached-combobox-v6/sessions/1779027970415-65904/share/1779027983606.zip`
  - promoted Resizable suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-resizable --dir target/fret-diag-resizable-suite-after-moving-cached-combobox-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - promoted Resizable suite result:
    `target/fret-diag-resizable-suite-after-moving-cached-combobox-v1/sessions/1779029073205-16452/suite.summary.json`
    reports `status=passed`, 2/2 rows, `scripts_with_evidence=2`,
    `overlay_chosen_side_counts.top=2`, and zero lint errors/warnings.
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
  - current runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-platform-preferences-runtime-environment-mutation.json --dir target/fret-diag-platform-preferences-runtime-environment-mutation-v2 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - current runtime result:
    passed; run id `1779029357027`.
  - current runtime evidence:
    `target/fret-diag-platform-preferences-runtime-environment-mutation-v2/sessions/1779029342505-133048/script.result.json`
  - current AI packet:
    `target/fret-diag-platform-preferences-runtime-environment-mutation-v2/sessions/1779029342505-133048/1779029357027/ai.packet`
  - current packed evidence:
    `target/fret-diag-platform-preferences-runtime-environment-mutation-v2/sessions/1779029342505-133048/share/1779029357027.zip`
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
- Command strict diagnostics authoring:
  - invariant:
    promoted Command scripts must prove the owning Command page before using page-local
    `ui-gallery-command-*` selectors, and long-page content clicks must use stable clicks with
    target-level window visibility proof.
  - findings:
    enabling strict lint for the promoted Command suite found 4 missing page-entry proofs, 20 plain
    long-page content clicks, and 6 stable-click targets that depended on nearby/root visibility
    instead of target-level visibility. These were harness authoring gaps; the hardened Command
    suite did not reproduce a new component or `fret-ui` mechanism defect.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`,
    `tools/diag-scripts/suites/ui-gallery-command/suite.json`,
    and promoted Command scripts under `tools/diag-scripts/ui-gallery/command/`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`,
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 18 tests.
  - full-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-command --dir target/fret-diag-command-suite-strict-authoring-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    `target/fret-diag-command-suite-strict-authoring-v1/sessions/1778966586171-88944/suite.summary.json`
    reports `status=passed`, 18/18 rows, `scripts_with_evidence=18`, and
    `focus_mismatch_total=0`.
- DataTable runtime semantics lint cleanup:
  - invariant:
    runtime evidence suites must catch visual-pass semantics drift: same-page recipe instances need
    unique diagnostics ids, and interactive button action owners need accessible names on the action
    node itself.
  - findings:
    `ui-gallery-shadcn-runtime-evidence` passed the DataTable pagination runtime assertions but
    failed suite lint on duplicate `data-table-toolbar-column-filter-input` ids and missing labels
    on table header sort buttons.
  - implementation anchors:
    `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`,
    `ecosystem/fret-ui-kit/src/declarative/table.rs`,
    `ecosystem/fret-ui-shadcn/tests/data_table_toolbar_global_filter.rs`,
    UI Gallery DataTable snippets under `apps/fret-ui-gallery/src/ui/snippets/data_table/`,
    and retained/view-cache DataTable scripts under `tools/diag-scripts/ui-gallery/data-table/`.
  - focused gates:
    `cargo test --profile dev-fast -p fret-ui-shadcn --test data_table_toolbar_global_filter data_table_toolbar_test_id_prefix_scopes_owned_inputs -- --nocapture`,
    `cargo test --profile dev-fast -p fret-ui-kit --lib table_virtualized_sort_header_button_exposes_accessible_label -- --nocapture`,
    and
    `cargo test --profile dev-fast -p fret-ui-kit --lib table_virtualized_retained_header_debug_ids_click_sort_actions -- --nocapture`.
  - focused gate results:
    passed.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - focused runtime gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-default-pagination-collection-metadata.json --dir target/fret-diag-data-table-default-pagination-lint-fix-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed with run id `1778969776320`.
  - suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target/fret-diag-shadcn-runtime-evidence-after-datatable-lint-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - suite result:
    `target/fret-diag-shadcn-runtime-evidence-after-datatable-lint-v1/sessions/1778969813032-104196/suite.summary.json`
    reports `status=passed`, 10/10 rows, `scripts_with_evidence=10`, and
    `focus_mismatch_total=0`.
- Fixed-frame-clock diagnostics contract:
  - invariant:
    promoted scripts that assert motion, transition, or delay outcomes with frame-count waits must
    pin the diagnostics frame clock, otherwise native runner scheduling can turn a small
    `wait_frames` interval into a different wall-clock contract.
  - findings:
    the HoverCard `trigger-delays` gate produced a false component signal because it relied on an
    unpinned frame count for a millisecond delay contract.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`,
    and `tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-trigger-delays.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`,
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 21 tests.
  - focused runtime gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-trigger-delays.json --dir target/fret-diag-hover-card-trigger-delays-fixed-delta-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed with run id `1778971779541`; launch env included
    `FRET_DIAG_FIXED_FRAME_DELTA_MS`.
  - suite follow-up:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-hover-card --dir target/fret-diag-hover-card-fixed-delta-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - suite follow-up result:
    first three HoverCard rows passed, including `trigger-delays`; the remaining failure is a
    separate `sides-placement` script authoring issue at
    `target/fret-diag-hover-card-fixed-delta-suite-v1/sessions/1778971816718-107892/suite.summary.json`.
- HoverCard sides placement oracle:
  - invariant:
    overlay placement scripts must encode the actual geometry precondition they create. If the
    trigger is near a collision boundary, the oracle should assert the collision flip rather than a
    preferred-side placement that cannot fit.
  - findings:
    `ui-gallery-hover-card-sides-placement.json` first failed because it moved the pointer to
    absent `ui-gallery-status-last-action`; after switching to a stable leave target, the bottom
    side check revealed an incorrect oracle. The bottom trigger had only ~43px of preferred-side
    space for a 120px panel, so the correct outcome is `chosen_side=top`, `flipped=true`.
  - implementation anchor:
    `tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-sides-placement.json`.
  - focused runtime gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-sides-placement.json --dir target/fret-diag-hover-card-sides-placement-flip-oracle-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed with run id `1778972701843`.
  - full-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-hover-card --dir target/fret-diag-hover-card-suite-after-sides-placement-oracle-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    `target/fret-diag-hover-card-suite-after-sides-placement-oracle-v1/sessions/1778972760323-23244/suite.summary.json`
    reports `status=passed`, 6/6 rows, `scripts_with_evidence=6`, and
    `focus_mismatch_total=0`.
- HoverCard strict diagnostics authoring gate:
  - invariant:
    promoted HoverCard scripts should not rediscover page-entry or long-page click precondition
    mistakes at runtime. Page-local `ui-gallery-hover-card-*` / `ui-gallery-hovercard-*` selectors
    require an owning page proof or `FRET_UI_GALLERY_START_PAGE=hover_card`, and stable clicks on
    those content targets require a prior visibility guard.
  - finding:
    dry-running the stricter registry rules over `ui-gallery-hover-card` found zero remaining
    violations after the earlier fixed-frame-clock and sides-placement repairs. This slice turned
    that clean state into a durable registry gate; no new HoverCard recipe or mechanism defect was
    reproduced.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`, and
    `tools/diag-scripts/suites/ui-gallery-hover-card/suite.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 24 tests and the promoted registry is up to date.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-hover-card --dir target/fret-diag-hover-card-strict-authoring-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime suite result:
    `target/fret-diag-hover-card-strict-authoring-v1/sessions/1779002136522-139728/suite.summary.json`
    reports `status=passed`, 6/6 rows, `scripts_with_evidence=6`,
    `focus_mismatch_total=0`, `reason_code_counts={}`, zero lint errors/warnings for every row,
    and overlay placement traces with chosen sides `left=3`, `right=1`, `top=5`.
- Menubar submenu placement focused suite:
  - invariant:
    Menubar submenu placement must cover LTR physical-right placement, RTL physical-left placement,
    and RTL tight-left collision flip behavior as a small independently runnable runtime gate.
  - findings:
    the three Menubar placement scripts were already useful, but only broad suites reached them.
    This was a harness packaging gap rather than a recipe or mechanism defect.
  - implementation anchors:
    `tools/diag-scripts/suites/ui-gallery-menubar-placement/suite.json`,
    `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-submenu-placement-trace.json`,
    `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-rtl-submenu-placement-trace.json`,
    and
    `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-rtl-submenu-tight-left-collision.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`,
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 21 tests.
  - full-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-menubar-placement --dir target/fret-diag-menubar-placement-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    `target/fret-diag-menubar-placement-suite-v1/sessions/1778973313432-109176/suite.summary.json`
    reports `status=passed`, 3/3 rows, `scripts_with_evidence=3`,
    `focus_mismatch_total=0`, and zero lint errors/warnings for all rows.
- Menubar Placement strict diagnostics authoring gate:
  - invariant:
    promoted Menubar Placement scripts should not rediscover page-entry or long-page click
    precondition mistakes at runtime. Page-local `ui-gallery-menubar-*` selectors require an
    owning page proof or `FRET_UI_GALLERY_START_PAGE=menubar`, and stable clicks on those content
    targets require a prior visibility guard.
  - finding:
    dry-running the stricter registry rules over `ui-gallery-menubar-placement` found zero current
    violations. The existing scripts already enter the Menubar page explicitly and guard their
    content clicks. A parallel DropdownMenu candidate dry-run found two remaining click-visibility
    gaps; those were handled separately in the DropdownMenu strict diagnostics authoring gate.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`, and
    `tools/diag-scripts/suites/ui-gallery-menubar-placement/suite.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 27 tests and the promoted registry is up to date.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-menubar-placement --dir target/fret-diag-menubar-placement-strict-authoring-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime suite result:
    `target/fret-diag-menubar-placement-strict-authoring-v1/sessions/1779003319016-117380/suite.summary.json`
    reports `status=passed`, 3/3 rows, run ids `1779003333926`, `1779003389077`, and
    `1779003426248`, with zero lint errors/warnings for every row.
- DropdownMenu strict diagnostics authoring gate:
  - invariant:
    promoted DropdownMenu scripts should not rediscover page-entry or long-page click precondition
    mistakes at runtime. Page-local `ui-gallery-dropdown-menu-*` selectors require an owning page
    proof or `FRET_UI_GALLERY_START_PAGE=dropdown_menu`, and stable clicks on those content targets
    require a prior visibility guard.
  - findings:
    the candidate strict dry-run found two click-visibility gaps. The focusable-disabled gate
    clicked `ui-gallery-dropdown-menu-demo-trigger.chrome` without first proving window
    containment, and the submenu smoke gate scrolled
    `ui-gallery-dropdown-menu-submenu-trigger.chrome` without setting
    `require_fully_within_window=true`. These were diagnostics authoring gaps, not DropdownMenu
    recipe defects.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`,
    `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json`,
    `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-submenu-open-smoke.json`,
    and `tools/diag-scripts/suites/ui-gallery-dropdown-menu/suite.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 30 tests and the promoted registry is up to date.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - focused rerun after first suite stall:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-basic-typeahead-billing.json --dir target/fret-diag-dropdown-menu-basic-typeahead-strict-rerun-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused rerun result:
    passed with run id `1779004715484`.
  - runtime suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-dropdown-menu --dir target/fret-diag-dropdown-menu-strict-authoring-v2 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime suite result:
    `target/fret-diag-dropdown-menu-strict-authoring-v2/sessions/1779004799053-142764/suite.summary.json`
    reports `status=passed`, 3/3 rows, `scripts_with_evidence=3`, `focus_mismatch_total=0`,
    `reason_code_counts={}`, and zero lint errors/warnings for every row.
  - first-attempt note:
    `target/fret-diag-dropdown-menu-strict-authoring-v1/sessions/1779004410147-126432/suite.summary.json`
    failed with `timeout.no_frames` in the Basic typeahead script after resize. Because the
    focused rerun and the full v2 suite passed, this remains a harness/run stability observation
    rather than a confirmed recipe or mechanism defect.
- ContextMenu strict diagnostics authoring gate:
  - invariant:
    promoted ContextMenu scripts should not rediscover page-entry or long-page click precondition
    mistakes at runtime. Page-local `ui-gallery-context-menu-*` selectors require an owning page
    proof or `FRET_UI_GALLERY_START_PAGE=context_menu`, and stable clicks on those content targets
    require a prior visibility guard.
  - finding:
    dry-running the stricter registry rules over `ui-gallery-context-menu` found zero current
    violations. The two corridor scripts already provide an explicit ContextMenu page default,
    assert `ui-gallery-page-context-menu`, and guard the submenu trigger with window containment
    before `click_stable`.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`, and
    `tools/diag-scripts/suites/ui-gallery-context-menu/suite.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 33 tests and the promoted registry is up to date.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - runtime suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-context-menu --dir target/fret-diag-context-menu-strict-authoring-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime suite result:
    `target/fret-diag-context-menu-strict-authoring-v1/sessions/1779005731883-139360/suite.summary.json`
    reports `status=passed`, 2/2 rows, `scripts_with_evidence=2`, `focus_mismatch_total=0`,
    `reason_code_counts={}`, overlay `chosen_side_counts.right=2`, and zero lint errors/warnings
    for every row.
- Button Group strict diagnostics authoring gate:
  - invariant:
    promoted Button Group scripts should not rediscover page-entry or long-page click precondition
    mistakes at runtime. Page-local `ui-gallery-button-group-*` selectors require an owning page
    proof or `FRET_UI_GALLERY_START_PAGE=button_group`, and stable clicks on those content targets
    require a prior visibility guard.
  - findings:
    the strict dry run found three unguarded Code-tab clicks in the Demo, Accessibility, and Select
    screenshot scripts. After converting them to `bounds_within_window` plus `click_stable`, the
    first runtime suite still failed the Select path because the Code tab selector was present but
    off-window (`y=2522.6665` in a `720px` window). The Select script now scrolls the section into
    the Gallery content viewport before both the Preview and Code captures.
  - implementation anchors:
    `tools/check_diag_scripts_registry.py`,
    `tools/test_check_diag_scripts_registry.py`,
    `tools/diag-scripts/ui-gallery/button/ui-gallery-button-group-demo-screenshots.json`,
    `tools/diag-scripts/ui-gallery/button/ui-gallery-button-group-accessibility-screenshots.json`,
    `tools/diag-scripts/ui-gallery/button/ui-gallery-button-group-select-screenshots.json`, and
    `tools/diag-scripts/suites/ui-gallery-button-group/suite.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 36 tests and the promoted registry is up to date.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - focused Select gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/button/ui-gallery-button-group-select-screenshots.json --dir target/fret-diag-button-group-select-strict-authoring-rerun-v1 --session-auto --pack --ai-packet --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused Select result:
    passed with run id `1779008006425`; AI packet:
    `target/fret-diag-button-group-select-strict-authoring-rerun-v1/sessions/1779007993048-40528/1779008006425/ai.packet`.
  - first strict suite attempt:
    `target/fret-diag-button-group-strict-authoring-v1/sessions/1779006929575-117696/suite.summary.json`
    failed at step 15 in `ui-gallery-button-group-select-screenshots.json` with
    `wait_until_timeout` on `ui-gallery-button-group-select-tabs-trigger-code`.
  - runtime suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-button-group --dir target/fret-diag-button-group-strict-authoring-v2 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - runtime suite result:
    `target/fret-diag-button-group-strict-authoring-v2/sessions/1779008052527-138688/suite.summary.json`
    reports `status=passed`, 13/13 rows, `scripts_with_evidence=13`,
    `focus_mismatch_total=0`, `reason_code_counts={}`, and zero lint errors/warnings for every row.
- DropdownMenu focused suite:
  - invariant:
    DropdownMenu runtime evidence should prove submenu placement, keyboard typeahead commit, and
    disabled-but-focusable suppression without depending on optional status-bar UI.
  - findings:
    the first Basic typeahead run found harness issues rather than a recipe defect. The script let
    the trigger sit at the window bottom edge before `click_stable`, and then used the optional
    `ui-gallery-status-last-action` semantics node as the result oracle even though the app
    snapshot already recorded `/shell/last_action = "menu.dropdown.orange"`.
  - implementation anchors:
    `tools/diag-scripts/suites/ui-gallery-dropdown-menu/suite.json`,
    `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-submenu-open-smoke.json`,
    `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-basic-typeahead-billing.json`,
    and
    `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-focusable-disabled-keyboard-suppression.json`.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - focused typeahead gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-basic-typeahead-billing.json --dir target/fret-diag-dropdown-typeahead-app-snapshot-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused typeahead result:
    passed with run id `1778974969846`.
  - full-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-dropdown-menu --dir target/fret-diag-dropdown-menu-suite-v2 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    `target/fret-diag-dropdown-menu-suite-v2/sessions/1778975019728-11100/suite.summary.json`
    reports `status=passed`, 3/3 rows, `scripts_with_evidence=3`,
    `focus_mismatch_total=0`, and zero lint errors/warnings for all rows.
- ContextMenu focused suite:
  - invariant:
    ContextMenu runtime evidence should prove safe-corridor pointer movement and branch/corridor
    submenu routing as a small independently runnable pointer-policy gate.
  - findings:
    the two ContextMenu corridor scripts were already useful, but only broad suites reached them.
    This was a harness packaging gap rather than a ContextMenu recipe or hit-test mechanism defect.
  - implementation anchors:
    `tools/diag-scripts/suites/ui-gallery-context-menu/suite.json`,
    `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-safe-corridor-sweep.json`,
    and
    `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-branch-corridor-routing.json`.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`,
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 21 tests.
  - full-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-context-menu --dir target/fret-diag-context-menu-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    `target/fret-diag-context-menu-suite-v1/sessions/1778975671673-115352/suite.summary.json`
    reports `status=passed`, 2/2 rows, and zero lint errors/warnings for both rows.
- ViewCache cached model mutation runtime companion:
  - invariant:
    ViewCache runtime evidence should prove model changes inside cached UI Gallery content through
    structured app-snapshot state, and strict lint should catch semantics drift that a visual-only
    assertion would miss.
  - findings:
    adding `/view_cache` snapshot fields and the focused `ui-gallery-view-cache` suite did not
    reproduce a ViewCache invalidation defect. The first strict suite run instead exposed a real
    shadcn Textarea recipe semantics defect: the pointer-only resize grip was exported as an
    unlabeled visible Button. The fix keeps the TextArea label visible while hiding the resize grip
    from the visible accessibility tree and removing it from Tab traversal.
  - implementation anchors:
    `apps/fret-ui-gallery/src/driver/runtime_driver.rs`,
    `apps/fret-ui-gallery/src/driver/window_bootstrap.rs`,
    `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
    `ecosystem/fret-ui-shadcn/src/textarea.rs`,
    `tools/check_diag_scripts_registry.py`,
    `tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-model-mutation-through-cache.json`,
    and
    `tools/diag-scripts/suites/ui-gallery-view-cache/suite.json`.
  - focused component semantics gate:
    `cargo test --profile dev-fast -p fret-ui-shadcn --lib textarea_resize_handle_stays_out_of_visible_accessibility_tree -- --nocapture`
  - focused component semantics result:
    passed.
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`,
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 21 tests.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - full-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-view-cache --dir target/fret-diag-view-cache-model-mutation-v2 --session-auto --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - full-suite result:
    `target/fret-diag-view-cache-model-mutation-v2/sessions/1778978131681-113548/suite.summary.json`
    reports `status=passed`, 1/1 row, `scripts_with_evidence=1`,
    `focus_mismatch_total=0`, and zero lint errors/warnings.
- Button Group strict diagnostics lint promotion:
  - invariant:
    The Button Group family already covers the originally reported visual/layout risk areas, so the
    focused suite should also reject future accessibility, duplicate-id, zero-size, and related
    diagnostics lint drift.
  - findings:
    the candidate strict run did not reproduce a new Button Group component or mechanism defect.
    All 13 scripts passed with zero lint errors/warnings and no focus mismatches, so the suite is
    now safe to run with `max_warning_issues=0`.
  - implementation anchors:
    `tools/diag-scripts/suites/ui-gallery-button-group/suite.json`
  - lint gates:
    `python tools/test_check_diag_scripts_registry.py`,
    `python tools/check_diag_scripts_registry.py`
  - lint results:
    passed; registry self-tests ran 21 tests.
  - candidate full-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-button-group --dir target/fret-diag-button-group-strict-candidate-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - candidate full-suite result:
    `target/fret-diag-button-group-strict-candidate-v1/sessions/1778978465500-116384/suite.summary.json`
    reports `status=passed`, 13/13 rows, `scripts_with_evidence=13`,
    `focus_mismatch_total=0`, and zero lint errors/warnings for every row.
- Button Group size icon-only Add geometry hardening:
  - invariant:
    icon-only Add controls should not rely on screenshots alone; each icon must keep a stable size
    and remain centered inside its button across the small, medium, and large variants.
  - findings:
    the first draft used the wrong selector assumption for the icon anchor. The real ids are
    `*-add-icon`, so the geometry proof only became valid after aligning to the actual test ids.
  - implementation anchors:
    `tools/diag-scripts/ui-gallery/button/ui-gallery-button-group-size-screenshots-zinc-light-dark.json`
  - run results:
    the focused geometry assertions passed inline with the existing Button Group family evidence;
    no new component or mechanism defect was reproduced.
- Carousel embla-engine strict diagnostics lint promotion:
  - invariant:
    carousel runtime evidence should be split into compact, independently runnable evidence units
    rather than relying on one wide docs-parity suite. The embla-engine sub-suite should reject
    future diagnostics lint drift for inertia, touch, resize reInit, loop continuity, and loop
    downgrade behavior.
  - findings:
    the wide `ui-gallery-carousel-docs-parity` candidate run exceeded the outer command timeout
    before writing a normal `suite.summary.json`, even though completed rows were clean and the
    focused autoplay stop-on-last-snap script passed independently. This was a harness packaging
    issue rather than a confirmed Carousel mechanism defect. The smaller
    `ui-gallery-carousel-embla-engine` suite passed normally and is the durable evidence unit for
    this slice.
  - implementation anchors:
    `tools/diag-scripts/suites/ui-gallery-carousel-embla-engine/suite.json`,
    `tools/diag-scripts/ui-gallery-carousel-demo-inertia-pixels-changed.json`,
    `tools/diag-scripts/ui-gallery-carousel-demo-inertia-touch-pixels-changed.json`,
    `tools/diag-scripts/ui-gallery-carousel-demo-reinit-resize-gate.json`,
    `tools/diag-scripts/ui-gallery-carousel-loop-continuity-touch-gate.json`,
    and
    `tools/diag-scripts/ui-gallery-carousel-loop-downgrade-cannot-loop-gate.json`.
  - focused autoplay gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-stop-on-last-snap-gate.json --dir target/fret-diag-carousel-last-snap-candidate-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused autoplay result:
    passed with run id `1778981149388`.
  - candidate sub-suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-carousel-embla-engine --dir target/fret-diag-carousel-embla-engine-strict-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - candidate sub-suite result:
    `target/fret-diag-carousel-embla-engine-strict-v1/sessions/1778982359710-115076/suite.summary.json`
    reports `status=passed`, 5/5 rows, `scripts_with_evidence=5`,
    `focus_mismatch_total=0`, and zero lint errors/warnings for every row.
- Date Picker strict diagnostics precondition hardening:
  - invariant:
    responsive and long-page Date Picker diagnostics should fail with actionable harness reasons,
    not generic stuck scrolling, and promoted Date Picker scripts should be independently runnable
    instead of depending on suite-only environment setup.
  - findings:
    the mobile Drawer script initially used a 480px window that selected the component's mobile
    branch but left the desktop Gallery shell sidebar visible, so the remaining content viewport
    was too narrow to fully contain the 240px trigger. The range-roving script also clicked an
    offscreen trigger directly and depended on suite-injected environment variables when run alone.
    These were diagnostics harness/script precondition gaps, not confirmed Date Picker component or
    `fret-ui` layout mechanism defects.
  - implementation anchors:
    `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_scroll.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`,
    `tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-dropdowns-mobile-drawer.json`,
    `tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-range-roving-skips-disabled.json`,
    and `tools/diag-scripts/suites/ui-gallery-date-picker/suite.json`.
  - focused harness gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics scroll_unscrollable_axis --no-fail-fast`
  - focused harness result:
    passed; Nextest run id `007a4bef-f826-4667-89bd-1a7cd5d41b12`.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - build result:
    passed.
  - focused mobile Drawer gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-dropdowns-mobile-drawer.json --dir target/fret-diag-date-picker-mobile-drawer-fix-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 360000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused mobile Drawer result:
    passed with run id `1778984155994`.
  - first strict suite result:
    `target/fret-diag-date-picker-strict-v2/sessions/1778984222427-88980/suite.summary.json`
    failed only on `ui-gallery-date-picker-range-roving-skips-disabled`, proving the remaining
    issue was a script precondition rather than the mobile Drawer path.
  - focused range-roving gate:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-range-roving-skips-disabled.json --dir target/fret-diag-date-picker-range-roving-scroll-fix-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused range-roving result:
    passed with run id `1778985968416`.
  - full Date Picker suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-date-picker --dir target/fret-diag-date-picker-strict-v3 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - full Date Picker suite result:
    `target/fret-diag-date-picker-strict-v3/sessions/1778986003617-126604/suite.summary.json`
    reports `status=passed`, 4/4 rows.
  - registry gates:
    `python tools/test_check_diag_scripts_registry.py`
    `python tools/check_diag_scripts_registry.py`
  - registry results:
    passed; registry self-tests ran 21 tests.
- Combobox geometry/placement focused suite:
  - invariant:
    Combobox visual-geometry and overlay-placement regressions should have a compact daily gate
    that covers trigger chrome, long-text truncation, chevron/checkmark spacing, top/bottom popup
    placement, and responsive resize placement without relying on the broad 24-script family suite.
  - findings:
    the broad `ui-gallery-combobox` candidate exceeded the outer command timeout before writing a
    normal `suite.summary.json`, even after many geometry/placement rows had passed. This was a
    harness packaging issue rather than a confirmed Combobox recipe or `fret-ui` overlay mechanism
    defect. The compact focused suite passed normally and is the better evidence unit for the
    layout questions that originally motivated the Combobox checks.
  - implementation anchors:
    `tools/diag-scripts/suites/ui-gallery-combobox-geometry-placement/suite.json` and
    `tools/diag-scripts/index.json`.
  - registry gates:
    `python tools/test_check_diag_scripts_registry.py`
    `python tools/check_diag_scripts_registry.py`
  - registry results:
    passed; registry self-tests ran 21 tests.
  - focused suite gate:
    `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target/fret-diag-combobox-geometry-placement-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused suite result:
    `target/fret-diag-combobox-geometry-placement-v1/sessions/1778988226668-109764/suite.summary.json`
    reports 7/7 passed rows, `scripts_with_evidence=7`, `focus_mismatch_total=0`, top placement
    traces for 3 rows, bottom placement traces for 4 rows, and zero lint errors/warnings for every
    row.
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

## Layout Primitive HoverRegion Absolute-Child Envelope Gate

- invariant:
  `HoverRegion` is a mechanism wrapper whose hover/hit-test envelope must include
  absolute-positioned children; its intrinsic measurement path must not collapse to `0 x 0` when
  the same final layout and hit-test paths keep the absolute child visible and targetable.
- finding:
  the tracer fixture first failed because the absolute child had visible bounds at `12,8 20x10`,
  but the HoverRegion layout bounds and `measure_in(MaxContent)` metrics were both `0 x 0`.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/measure.rs`,
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`, and
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-capture`
- result:
  passed after adding the dedicated HoverRegion measurement path and a center hit-test assertion for
  `hover-region-absolute-child` (latest run id:
  `e3424dfe-3295-4819-8f1b-8e10f02eb77d`).

## Layout Primitive HoverRegion Fractional-Inset Envelope Gate

- invariant:
  `HoverRegion` absolute-child hover/hit-test envelopes must account for fractional insets during
  shrink-wrap sizing, not only fixed pixel insets.
- finding:
  the fractional tracer fixture first failed because the wrapper and `measure_in(MaxContent)` stayed
  at the child size `20 x 10`, while final placement resolved `left: 25%` and `top: 10%` and pushed
  the child outside the wrapper envelope.
- implementation anchors:
  `crates/fret-ui/src/declarative/layout_helpers.rs`,
  `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`,
  `crates/fret-ui/src/declarative/host_widget/measure.rs`,
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`, and
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-capture`
- result:
  passed after adding the shared conservative fractional envelope helper and a near-edge hit-test
  assertion for `hover-region-fractional-child` (latest run id:
  `b3093496-7227-441c-87d7-608cf7bd97c3`).
- right/bottom companion:
  real recipe surfaces place scrollbar chrome under HoverRegion wrappers with right/bottom absolute
  insets (`ecosystem/fret-ui-shadcn/src/scroll_area.rs` and
  `ecosystem/fret-code-view/src/code_block.rs`). The companion
  `hover-region-right-bottom-inset-envelope-matches-layout` fixture locks the same layout/measure
  envelope plus a near-edge hit-test sample. It passed without a new mechanism change with run id
  `4ab0b09a-f343-4d37-89d9-646b00bf491c`.
- adjacent regression gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui declarative::tests::layout::basics --no-capture`
  passed 38/38 with run id `c394dc00-75ad-472c-8ac3-303eb9745667`, and
  `cargo nextest run --cargo-profile dev-fast -p fret-ui declarative::tests::layout::interactivity --no-capture`
  passed 17/17 with run id `e3f5be40-3ea4-4665-ba08-0d412f1d792e`.
- viewport-root wrapper regression gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui declarative::tests::layout::viewport_roots --no-capture`
  passed 37/37 with run id `279a732e-d9da-4033-9701-3e3ccef1e05b`.

## VirtualList Auto-Height Measured-Leaf and AI FileTree Runtime Gate

- invariant:
  an auto-height `VirtualList` used as a Taffy measured leaf must remeasure its parent layout when
  layout-affecting VirtualList props such as `len` or `items_revision` change. Updated semantics
  and child rows are not enough; following document sections must move out of the expanded list's
  hit-test area.
- finding:
  the AI FileTree semantics/action-state suite found the expanded `file-lib` row in the semantics
  tree, but the row was outside the stale FileTree root height and was overlapped by the next Basic
  Usage doc section. This was a real `fret-ui` mechanism defect in measured-leaf dirtying, not a
  retained-host defect, because AI FileTree uses `cx.virtual_list_keyed_with_layout`.
- implementation anchors:
  `crates/fret-ui/src/declarative/mount.rs`,
  `crates/fret-ui/src/layout/engine.rs`,
  `crates/fret-ui/src/layout/engine/flow.rs`, and
  `crates/fret-ui/src/declarative/tests/virtual_list/measurement.rs`.
- diagnostics anchors:
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-file-tree-demo-toggle.json`,
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-file-tree-demo-actions.json`, and
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-file-tree-large-scroll.json`.
- focused regression gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui auto_height_virtual_list_len_growth_reflows_following_siblings --no-fail-fast --no-capture`
  - result: passed.
- VirtualList family gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui virtual_list --no-fail-fast`
  - result: passed, 50/50 tests.
- formatting gate:
  `cargo fmt -p fret-ui --check`
  - result: passed.
- registry gates:
  `python tools/check_diag_scripts_registry.py`
  `python tools/test_check_diag_scripts_registry.py`
  - result: passed; registry self-tests ran 36 tests.
- runtime suite gate:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-ai-file-tree --dir target\fret-diag-ai-file-tree-semantics-action-state-after-vlist-measured-leaf-dirty-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev`
  - result:
    `target/fret-diag-ai-file-tree-semantics-action-state-after-vlist-measured-leaf-dirty-v1/sessions/1779041323318-52900/suite.summary.json`
    reports `status=passed`, 4/4 rows.
  - row run ids:
    `toggle=1779041409356`,
    `actions=1779041435085`,
    `large-scroll=1779041463491`, and
    `screenshot=1779041561485`.
- strict zero-warning follow-up:
  demo-only `0 x 0` state markers are now hidden semantics anchors, the scripts assert them through
  `raw_semantics_hidden_is`, and `fret-diag` ignores visible-bounds/missing-label lint for
  non-focused hidden nodes while preserving lint for visible zero-size test-id nodes.
- focused lint regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag lint_ignores_hidden_state_anchors_for_visible_bounds_warnings --no-fail-fast --no-capture`
  - result: passed.
- rebuilt diagnostics/gallery binaries:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- strict runtime suite gate:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-ai-file-tree --dir target\fret-diag-ai-file-tree-zero-warning-hidden-markers-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev`
  - result:
    `target/fret-diag-ai-file-tree-zero-warning-hidden-markers-v2/sessions/1779043283276-169328/suite.summary.json`
    reports `status=passed`, 4/4 rows.
  - lint result:
    every row `check.lint.json` reports `error_issues=0`, `warning_issues=0`, and empty
    `counts_by_code`.
  - row run ids:
    `toggle=1779043310053`,
    `actions=1779043336823`,
    `large-scroll=1779043363896`, and
    `screenshot=1779043433309`.

## Layout Primitive Grid Gap Measurement Gate

- invariant:
  grid `column_gap` and `row_gap` must affect both final child placement and intrinsic
  `measure_in(MaxContent)` size. Otherwise auto-size parents and scroll extents can disagree with
  the visual grid.
- finding:
  the new fixture did not reproduce a `fret-ui` mechanism defect. The first red run found an oracle
  mistake: with first-row height `10` and `row_gap=6`, the second row starts at `y=16`, not `18`,
  and total height is `28`.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs` and
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
- result:
  passed after correcting the oracle; Nextest run id
  `f1be5c37-82c6-4ffe-b55a-f7f19090fd33`.

## Layout Primitive Flex Order Auto-Margin Gate

- invariant:
  flex `order` must define the visual child sequence for late flex layout post-processing as well
  as for the engine solve and intrinsic measurement. Auto-margin trailing-group alignment must not
  fall back to source-order siblings.
- finding:
  the `flex-order-auto-margin-uses-visual-order` fixture first failed because
  `layout_flex_impl_engine` scanned `cx.children` in source order while the solved flex row used
  visual order. In the row `A(order=2)`, `B(order=0, ml-auto)`, `C(order=1)`, child B landed at
  `x=40` instead of `10`, and child C landed at `x=80` instead of `50`.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/layout/flex.rs`,
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`, and
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
- result:
  passed after routing auto-margin tail detection, tail-size computation, gap preservation, shift
  application, and final child layout iteration through `ordered_flex_children`; Nextest run id
  `a53d39d8-f93e-4390-b859-28b233a843a1`.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.

## Layout Primitive Flex Gap Measurement Gate

- invariant:
  flex `gap` must affect both final child placement and intrinsic `measure_in(MaxContent)` size.
  Otherwise recipe rows/stacks can lay out correctly while auto-size parents or scroll extents use a
  smaller measured envelope.
- finding:
  the new `flex-gap-measure-matches-layout` fixture did not reproduce a new mechanism defect. It
  proves a two-child horizontal flex row with `gap=8` places child B at `x=28` and reports matching
  final and measured envelope metrics of `58 x 12`.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`,
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`,
  `crates/fret-ui/src/declarative/host_widget/measure.rs`, and
  `crates/fret-ui/src/layout/engine/flow.rs`.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
- result:
  passed; Nextest run id `4468bb08-582b-4be1-a09f-eb9e59149a41`.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.

## Layout Primitive Flex Order Margin-Top Auto Gate

- invariant:
  `margin-top: auto` in a vertical flex column must use the visual child sequence when it aligns a
  trailing group. Ordered vertical children must not regress to source-order tail detection.
- finding:
  the new `flex-order-margin-top-auto-uses-visual-order` fixture did not reproduce a new mechanism
  defect. It closes the vertical companion to F166 for real recipe surfaces that use `mt_auto()`,
  proving `B(order=0, mt-auto)`, `C(order=1)`, and `A(order=2)` land at `y=10`, `y=50`, and
  `y=70` in a 100px column.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`,
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`,
  `crates/fret-ui/src/declarative/tests/layout/basics.rs`,
  `ecosystem/fret-ui-shadcn/src/sheet.rs`, and `ecosystem/fret-ui-shadcn/src/drawer.rs`.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
- result:
  passed; Nextest run id `93063426-0c6d-4e44-bba0-fc0bb68de234`.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.

## Layout Primitive Flex Order Margin-Right Auto Gate

- invariant:
  `margin-right: auto` must remain consistent with flex visual order when recipes use it as an RTL
  logical auto margin. Combining `FlexItemStyle.order` with right-side auto margin must not leave
  later visual siblings in source-order positions.
- finding:
  the new `flex-order-margin-right-auto-uses-visual-order` fixture did not reproduce a new
  mechanism defect. It closes the right-side auto-margin companion to F166 and proves the current
  row layout places `B(order=0, mr-auto)`, `C(order=1)`, and `A(order=2)` at `x=0`, `x=50`, and
  `x=70` in a 100px row.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`,
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`,
  `ecosystem/fret-ui-shadcn/src/rtl.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/sidebar/app_sidebar.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/progress/rtl.rs`.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
- result:
  passed; Nextest run id `7592ea5c-a8fa-4a11-8869-8533db84bdfb`.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.

## Layout Primitive Flex Wrap Gap Measurement Gate

- invariant:
  wrapped flex `gap` must affect both final child placement and definite-width intrinsic
  `measure_in(MaxContent)` size. Otherwise multi-line recipe stacks can draw with correct line
  spacing while auto-size parents or scroll extents use a smaller measured envelope.
- finding:
  the new `flex-wrap-gap-measure-matches-layout` fixture did not reproduce a new mechanism defect.
  It proves a 68px-wide wrapping row with `gap=8` places child B at `x=38`, places child C on the
  second line at `y=20`, and reports matching final and measured envelope metrics of `68 x 34`.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`,
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`,
  `crates/fret-ui/src/declarative/host_widget/measure.rs`, and
  `crates/fret-ui/src/layout/engine/flow.rs`.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
- result:
  passed; Nextest run id `0353a271-aaaf-49d0-9254-f770af9ba4c1`.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.

## Layout Primitive Relative Inset Flow-Sibling Gate

- invariant:
  `PositionStyle::Relative` inset offsets must move the target's final layout and hit-test
  position without changing sibling flow placement. This locks ADR 0062's typed position/inset
  primitive and `element.rs`'s contract that relative inset offsets tweak final position without
  affecting siblings.
- finding:
  `relative-inset-offsets-final-position-without-affecting-flow-siblings` did not reproduce a new
  mechanism defect. It proves a `20 x 10` Pressable in a horizontal flex row with `top: 12px`
  lands at `0,12`, while the following `30 x 10` sibling stays at `20,0`. The original flow-slot
  center misses the moved Pressable and the final-position center hits it.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`,
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`,
  `crates/fret-ui/src/element.rs`,
  `crates/fret-ui/src/declarative/layout_helpers.rs`,
  `crates/fret-ui/src/declarative/host_widget/layout/flex.rs`, and
  `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`.
- JSON fixture validation:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\layout_primitives_v1.json`
  - result: passed.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
- result:
  passed; Nextest run id `6a87d598-b4f7-4b0d-83c6-c9842cdb9d25`.

## Layout Primitive Static Inset Ignore Gate

- invariant:
  `PositionStyle::Static` inset offsets must be ignored. Default flow-positioned nodes should keep
  their original flow slot, keep sibling placement unchanged, and route hit-testing through the
  flow slot rather than a hypothetical inset-offset position.
- finding:
  `static-inset-ignored-by-default-flow-position` did not reproduce a new mechanism defect. It
  proves a `20 x 10` Pressable with `top: 12px` but no positioned mode stays at `0,0`, keeps the
  following `30 x 10` sibling at `20,0`, hits at the original flow-slot center, and does not hit at
  the hypothetical `top: 12px` offset center.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`,
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`,
  `crates/fret-ui/src/element.rs`,
  `crates/fret-ui/src/declarative/taffy_layout.rs`,
  `crates/fret-ui/src/layout/engine/flow.rs`, and
  `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`.
- JSON fixture validation:
  `python -m json.tool crates\fret-ui\src\declarative\tests\fixtures\layout_primitives_v1.json`
  - result: passed.
- gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
- result:
  passed; Nextest run id `63fb7f75-45f1-4f9b-bbfa-4f20d22d7d5c`.

## Layout Primitive Pressable Absolute-Only Wrapper Envelope Gate

- invariant:
  auto-sized behavioral passthrough wrappers with only absolute-positioned children must not
  collapse to `0 x 0` when the absolute child defines the visible and hit-testable envelope.
  Wrapper layout bounds, child placement, placeholder measurement, and hit-testing must agree.
- finding:
  `pressable-fractional-absolute-child-envelope-matches-layout` first failed because an auto/auto
  `Pressable` with one absolute child was solved as a `0 x 0` flow item. The child could be placed
  and hit-tested after manual absolute layout, but the parent wrapper's authoritative layout bounds
  stayed collapsed. The expected envelope for `left: 25%`, `top: 10%`, and a `20 x 10` child is
  `27 x 12`.
- implementation anchors:
  `crates/fret-ui/src/layout/engine/flow.rs`,
  `crates/fret-ui/src/layout/engine.rs`,
  `crates/fret-ui/src/declarative/host_widget/measure.rs`,
  `crates/fret-ui/src/declarative/host_widget/layout.rs`,
  `crates/fret-ui/src/declarative/layout_helpers.rs`,
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`, and
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`.
- first red gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  - result: failed; Nextest run id `9aab41b5-8444-4086-a1cb-38307cb1467b`.
- fixed gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  - result: passed; Nextest run id `cedf2113-d532-4c84-b69a-728b052ae6a0`.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.

## Layout Primitive Pressable Mixed Flow/Absolute Wrapper Envelope Gate

- invariant:
  auto-sized behavioral passthrough wrappers with both flow children and absolute-positioned
  children must size and hit-test against the union envelope. The flow child must keep its measured
  size, while the absolute child must be placed against the same union envelope used for wrapper
  bounds and placeholder measurement.
- finding:
  `pressable-mixed-flow-absolute-child-envelope-matches-layout` first failed because an auto/auto
  `Pressable` with a `20 x 10` flow child and a fractional-inset `25 x 10` absolute child used the
  flow child's envelope for wrapper layout and placeholder measurement. The absolute child was
  placed at `(5, 1)` in a too-small containing block and a near-edge hit-test landed on the wrapper
  instead of the absolute child. The correct union envelope is `34 x 12`, placing the absolute child
  at `(8.5, 1.2)`.
- implementation anchors:
  `crates/fret-ui/src/layout/engine/flow.rs`,
  `crates/fret-ui/src/declarative/host_widget/measure.rs`,
  `crates/fret-ui/src/declarative/host_widget/layout.rs`,
  `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`,
  `crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs`, and
  `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`.
- first red gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  - result: failed; Nextest run id `8ca82632-03d0-41b9-969b-127241a687c5`.
- fixed gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  - result: passed; Nextest run id `0f75010e-ebc0-4f4c-b835-aff6c0086b9d`.
- companion layout gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui declarative::tests::layout::basics declarative::tests::layout::interactivity --no-fail-fast`
  - result: passed, 55/55 tests.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.
- scoped whitespace:
  `git diff --check -- crates/fret-ui/src/declarative/host_widget/layout.rs crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs crates/fret-ui/src/declarative/host_widget/measure.rs crates/fret-ui/src/layout/engine/flow.rs crates/fret-ui/src/declarative/tests/layout/mechanism_harness.rs crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json`
  - result: passed.

## ViewCache Relative Inset Clean-Reuse Movement Gate

- invariant:
  clean ViewCache reuse must translate cached interaction records and current element bounds by the
  cache-root movement even when the cached child uses `PositionStyle::Relative` inset offsets. The
  relative inset still defines final-position hit space, and the old final-position center must not
  remain targetable after the cache root moves.
- finding:
  `view_cache_hit_moving_relative_inset_wrapper_updates_bounds_and_hit_test` did not reproduce a new
  mechanism defect. It proves a cached `20 x 10` Pressable with `top: 12px` renders once, then moves
  from `0,12` to `40,12` when an outer spacer changes from `0` to `40`. Layout bounds, visual
  bounds, fallback hit-testing, and `debug_hit_test_routing` agree on the moved final position.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/view_cache.rs`,
  `crates/fret-ui/src/elements/runtime.rs`,
  `crates/fret-ui/src/tree/prepaint/interaction.rs`,
  `crates/fret-ui/src/element.rs`, and
  `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`.
- focused gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_hit_moving_relative_inset_wrapper_updates_bounds_and_hit_test --no-fail-fast --no-capture`
- result:
  passed; Nextest run id `9db3ccd2-727f-4e22-be43-bd9f6f1f4b09`.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.

## Input Disabled TextInput Action-State Runtime Gate

- invariant:
  a disabled leaf TextInput must publish disabled semantics and suppress accessibility actions that
  would focus or mutate the value. Visual disabled styling alone is not enough; the concrete
  TextInput node must report `disabled=true`, `focus=false`, and `set_value=false`.
- finding:
  `ui-gallery-input-disabled-action-state.json` did not reproduce a new Input recipe defect. The UI
  Gallery disabled Input already exports the expected TextInput semantics action state. Early
  scroll-based drafts exposed a separate diagnostics authoring hazard: the long Input page
  `scroll_into_view` path did not reliably move the Disabled section before a bounds wait, so the
  promoted gate proves the action-state contract without depending on that scroll path.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/input/disabled.rs`,
  `tools/diag-scripts/ui-gallery/input/ui-gallery-input-disabled-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\input\ui-gallery-input-disabled-action-state.json > $null`
  - result: passed.
- roundtrip gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_input_disabled_action_state --no-fail-fast`
  - result: passed; Nextest run id `4317d185-d642-4d7b-a042-592ef62530ce`.
- build gate:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- runtime gate:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-disabled-action-state.json --dir target\fret-diag-input-disabled-action-state-v4 --session-auto --pack --ai-packet --include-screenshots --timeout-ms 360000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779094906772`.
  - artifacts:
    `target/fret-diag-input-disabled-action-state-v4/sessions/1779094885443-189836/script.result.json`,
    `target/fret-diag-input-disabled-action-state-v4/sessions/1779094885443-189836/1779094906772/ai.packet`, and
    `target/fret-diag-input-disabled-action-state-v4/sessions/1779094885443-189836/share/1779094906772.zip`.
- formatting:
  `cargo fmt -p fret-ui-gallery -p fret-diag-protocol --check`
  - result: passed.

## ViewCache Relative Inset Semantics Movement Gate

- invariant:
  clean ViewCache reuse must keep semantics nodes observable exactly once and must translate their
  semantics bounds when the cache root moves, even when the cached child uses
  `PositionStyle::Relative` inset offsets. The semantics snapshot must not retain stale
  final-position bounds from the old cache-root origin.
- finding:
  `view_cache_semantics_moving_relative_inset_updates_bounds_without_rerender` did not reproduce a
  new mechanism defect. The current runtime keeps one semantics node for the cached Pressable and
  moves its bounds from `0,12` to `40,12` while the cached render closure runs once.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/view_cache.rs`,
  `crates/fret-ui/src/elements/runtime.rs`,
  `crates/fret-ui/src/tree/ui_tree_semantics.rs`,
  `crates/fret-ui/src/tree/prepaint/interaction.rs`,
  `crates/fret-ui/src/element.rs`,
  `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`, and
  `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`.
- focused gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_semantics_moving_relative_inset_updates_bounds_without_rerender --no-fail-fast --no-capture`
  - result: passed; Nextest run id `c013f3b5-819d-45ba-8722-ddea5139213d`.
- formatting:
  `cargo fmt -p fret-ui --check`
  - result: passed.

## Retained Table Header Bounds Flex Snapshot Gate

- invariant:
  flex final layout must use a stable ordered child-rect snapshot while recursively laying out
  children. Recursive layout of an earlier child must not make later siblings appear unsolved to the
  same final flex pass.
- finding:
  the retained Table direct-start Gallery path exposed a real `fret-ui` mechanism defect. The
  retained Table subtree existed, but its header row stayed at `0,0 0x0` because
  `layout_flex_impl_engine` re-queried live sibling rects after recursive child layout invalidated
  later solved stamps. The fix snapshots child rects before recursion and uses the snapshot for
  auto-margin tail sizing, gap preservation, shift application, and final child layout.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/layout/flex.rs` and
  `apps/fret-ui-gallery/src/driver/render_flow.rs`.
- focused retained Table bounds gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --features gallery-dev table_retained_torture_direct_start_header_bounds_converge --no-fail-fast --no-capture`
  - result: passed; Nextest run id `e84b549f-2b87-4faa-afb2-969c294ae01e`.
- layout companion gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  - result: passed; Nextest run id `faa8f32d-8f3e-4831-aa95-00e1861f831b`.
- retained Table selected semantics companion:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_selected_semantics_follow_windowed_row_selection --no-fail-fast --no-capture`
  - result: passed; Nextest run id `9951e6c7-722f-4713-be3f-797dd2d01a6e`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_table_retained_sort_select_scroll script_v2_roundtrip_ui_gallery_table_retained_window_boundary_scroll --no-fail-fast`
  - result: passed; Nextest run id `f8df7c84-6a3e-4dde-bbbe-0c0e31546407`.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\table\ui-gallery-table-retained-sort-select-scroll.json --dir target\fret-diag-table-retained-selected-sort-select-scroll-after-flex-snapshot --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  - result: failed later at step 12 `selected_is true`, after the previous header-row
    `bounds_within_window` precondition passed.
  - artifact:
    `target/fret-diag-table-retained-selected-sort-select-scroll-after-flex-snapshot/sessions/1779113354850-74192/script.result.json`.
  - follow-up:
    click step 10 intended `ui-gallery-table-retained-row-0`, but hit-tested the enclosing
    `scroll_bar` node. Split that as a hit-test/routing or diagnostics click-targeting slice.

## Absolute Positioned Explicit-Size Hit Region Gate

- invariant:
  manual absolute-child layout must preserve a child's explicit `SizeStyle` when the child is
  positioned with one pinned edge on an axis. A `right: 0; width: 10px` scrollbar overlay must not
  expand to the full probe/viewport width and cover underlying content hits.
- finding:
  the retained Table runtime follow-up from F191 exposed a real `fret-ui` positioned-layout defect.
  `PositionedLayoutStyle::Absolute` carried only `InsetStyle`, so manual absolute layout paths laid
  out the shadcn ScrollArea scrollbar gate without its explicit width. Its fill-sized
  `Opacity -> Scrollbar` descendants expanded to the viewport and intercepted row clicks.
- implementation anchors:
  `crates/fret-ui/src/declarative/layout_helpers.rs`,
  `crates/fret-ui/src/declarative/host_widget/layout.rs`,
  `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`,
  `crates/fret-ui/src/declarative/tests/layout/scroll.rs`, and
  `tools/diag-scripts/ui-gallery/table/ui-gallery-table-retained-sort-select-scroll.json`.
- focused mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui absolute_interactivity_gate_preserves_scrollbar_track_bounds --no-fail-fast --no-capture`
  - result: passed; Nextest run id `ae114762-9f8e-4ac9-9594-606305eee7ec`.
- layout companion gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_layout_primitives_match_oracles --no-fail-fast --no-capture`
  - result: passed; Nextest run id `7163fc89-31dd-4f1d-a2ef-ba2e522dac41`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_table_retained_sort_select_scroll --no-fail-fast --no-capture`
  - result: passed; Nextest run id `67a19613-9301-4ef6-98a4-e20af5bff6b4`.
- runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\table\ui-gallery-table-retained-sort-select-scroll.json --dir target\fret-diag-table-retained-selected-sort-select-scroll-after-absolute-size-fix-current --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  - result: passed; run id `1779121343180`.
  - artifacts:
    `target/fret-diag-table-retained-selected-sort-select-scroll-after-absolute-size-fix-current/sessions/1779121262133-76528/1779121343180/ai.packet` and
    `target/fret-diag-table-retained-selected-sort-select-scroll-after-absolute-size-fix-current/sessions/1779121262133-76528/share/1779121343180.zip`.
- formatting/static checks:
  `cargo fmt -p fret-ui --check`
  - result: passed.
  `python -m json.tool tools\diag-scripts\ui-gallery\table\ui-gallery-table-retained-sort-select-scroll.json > $null`
  - result: passed.
  `git diff --check`
  - result: passed.

## Retained DataTable Selected-State Runtime Gate

- invariant:
  retained DataTable row selection must refresh `SemanticsNode.flags.selected` on the clicked row,
  and retained/window movement must not leak selected state into newly visible rows.
- finding:
  no new mechanism or recipe defect was reproduced. The retained DataTable torture page already
  keeps row-selection semantics fresh across the sort/select/scroll path after the retained Table
  scrollbar hit-region fix. This slice converts that risk into a durable diagnostics oracle.
- implementation anchors:
  `ecosystem/fret-ui-kit/src/declarative/table.rs`,
  `apps/fret-ui-gallery/src/ui/previews/gallery/data/table_torture.rs`,
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-sort-select-scroll.json`,
  `tools/diag-scripts/ui-gallery-data-table-retained-sort-select-scroll.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-sort-select-scroll.json > $null`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_data_table_retained_sort_select_scroll --no-fail-fast --no-capture`
  - result: passed; Nextest run id `a23c2c5e-e7a7-499b-b2c3-62b73ed5ffd8`.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- runtime diagnostics:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-sort-select-scroll.json --dir target\fret-diag-data-table-retained-selected-sort-select-scroll-current --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev`
  - result: passed; run id `1779122449287`; `tooling.warnings.json` is empty.
  - selected-state proof:
    script step 23 asserts `ui-gallery-data-table-row-0 selected=false`, step 27 asserts
    `ui-gallery-data-table-row-0 selected=true`, and step 48 asserts
    `ui-gallery-data-table-row-10015 selected=false`.
  - artifacts:
    `target/fret-diag-data-table-retained-selected-sort-select-scroll-current/sessions/1779122417901-101808/1779122449287/ai.packet`
    and
    `target/fret-diag-data-table-retained-selected-sort-select-scroll-current/sessions/1779122417901-101808/share/1779122449287.zip`.
- formatting/static checks:
  `cargo fmt -p fret-diag-protocol --check`
  - result: passed.
  `git diff --check`
  - result: passed; only Git's line-ending notice for `WORKSTREAM.json` was printed.

## Node Graph Cull Runtime Gate Coverage

- invariant:
  retained Node Graph cull windows must produce observable `node_graph_cull_window_shift` prepaint
  actions when panning far enough to cross cull-window boundaries, while small reversible pans
  should stay within the current cull window and produce zero cull-window shifts.
- finding:
  no new mechanism defect was reproduced. The promoted suites passed against the rebuilt
  `target/dev-fast/fret-ui-gallery.exe` binary. A previous `ensure_visible_timeout` came from a
  stale `target/debug/fret-ui-gallery.exe` binary: after nav search, selector resolution reported
  `match_count=0` for `ui-gallery-nav-node-graph-cull-torture`.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs`,
  `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window.rs`,
  `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window_shift.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/prepaint_diagnostics.rs`,
  `crates/fret-diag/src/diag_suite.rs`,
  `crates/fret-diag/src/stats/debug_stats_gates.rs`,
  `tools/diag-scripts/ui-gallery-node-graph-cull-torture-pan-zoom.json`,
  `tools/diag-scripts/ui-gallery-node-graph-cull-window-shifts.json`,
  `tools/diag-scripts/ui-gallery-node-graph-cull-window-no-shifts-small-pan.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_node_graph_cull --no-fail-fast --no-capture`
  - result: passed; Nextest run id `fad3a59e-43d7-47b4-9183-81ae290e61d5`.
- runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-node-graph-cull --dir target/fret-diag-node-graph-cull-suite-current --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; suite summary
    `target/fret-diag-node-graph-cull-suite-current/sessions/1779124193835-97600/suite.summary.json`;
    run id `1779124205290`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-node-graph-cull-window-shifts --dir target/fret-diag-node-graph-cull-window-shifts-suite-current --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; suite summary
    `target/fret-diag-node-graph-cull-window-shifts-suite-current/sessions/1779124242539-101504/suite.summary.json`;
    run id `1779124258887`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-node-graph-cull-window-no-shifts-small-pan --dir target/fret-diag-node-graph-cull-window-no-shifts-small-pan-suite-current --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; suite summary
    `target/fret-diag-node-graph-cull-window-no-shifts-small-pan-suite-current/sessions/1779124242622-103812/suite.summary.json`;
    run id `1779124253283`.
- stale-binary false-failure evidence:
  `target/fret-diag-node-graph-cull-suite/sessions/1779123851758-104128/script.result.json`
  - result: failed with `ensure_visible_timeout`; selector trace had `match_count=0` for
    `ui-gallery-nav-node-graph-cull-torture`. The same scripts passed after rebuilding and using
    `target/dev-fast/fret-ui-gallery.exe`.
- formatting/static checks:
  `cargo fmt -p fret-diag-protocol --check`
  - result: passed.
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
  `git diff --check`
  - result: passed.

## Canvas Cull Runtime Gate Stabilization

- invariant:
  Canvas Cull pan/zoom diagnostics must exercise the actual Gallery torture page and prove rendered
  pixels change after pan/zoom interaction. The script must not rely on off-window nav rows being
  directly clickable.
- finding:
  the first runtime suite failed before reaching Canvas Cull. Selector resolution found
  `ui-gallery-nav-canvas-cull-torture`, but the row was at `y=993.3` in a `720px` window; the
  click was clamped to the window edge, hit-tested `no_hit`, and the following `focus_is` wait
  timed out. This was a diagnostics authoring defect. After switching to nav search plus
  `ensure_visible`, the suite passed with zero lint warnings and pixels-changed evidence.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-canvas-cull-torture-pan-zoom.json`,
  `tools/diag-scripts/ui-gallery-canvas-cull-torture-pan-zoom.json`,
  `tools/diag-scripts/suites/ui-gallery-canvas-cull/suite.json`,
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/canvas_cull_torture.rs`,
  `crates/fret-diag/src/diag_suite.rs`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- first failed runtime evidence:
  `target/fret-diag-canvas-cull-suite-current/sessions/1779125762675-98156/script.result.json`
  - result: failed at step 1 with `wait_until_timeout`; hit trace for step 0 reported
    `clamped_outside_window=true`, `routing_explain="hit-test returned no node"`, and
    `intended_test_id="ui-gallery-nav-canvas-cull-torture"`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\perf\ui-gallery-canvas-cull-torture-pan-zoom.json > $null`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_canvas_cull_torture_pan_zoom --no-fail-fast --no-capture`
  - result: passed; Nextest run id `b32d1fd3-74f8-46c3-893f-1bee7b5d65f6`.
- runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-canvas-cull --dir target/fret-diag-canvas-cull-suite-after-search-entry --session-auto --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; suite summary
    `target/fret-diag-canvas-cull-suite-after-search-entry/sessions/1779125863873-16812/suite.summary.json`;
    run id `1779125873114`; lint warnings `0`.
  - pixels-changed post-run proof:
    `target/fret-diag-canvas-cull-suite-after-search-entry/sessions/1779125863873-16812/check.pixels_changed.json`.

## Gallery Nav Click Visibility Authoring Lint

- invariant:
  promoted UI Gallery scripts that click long navigation page rows must prove the nav row is inside
  the window first. Selector existence alone is not enough because off-window nav rows can resolve
  successfully while click synthesis clamps to the window edge and hit-tests `no_hit`.
- finding:
  the Canvas Cull runtime failure exposed a missing registry guard for `ui-gallery-nav-*` page-row
  clicks. Existing strict click-visibility lint covered long-page content targets, but not the left
  Gallery navigation list.
- implementation anchors:
  `tools/check_diag_scripts_registry.py` and `tools/test_check_diag_scripts_registry.py`.
- scope:
  strict nav-click visibility now covers the cleared cull/torture suites:
  `ui-gallery-canvas-cull`, `ui-gallery-chart-torture`, `ui-gallery-node-graph-cull`,
  `ui-gallery-node-graph-cull-window-shifts`, and
  `ui-gallery-node-graph-cull-window-no-shifts-small-pan`.
- registry self-test:
  `python tools/test_check_diag_scripts_registry.py`
  - result: passed; 39 tests.
- registry check:
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.

## Chart Torture Sampling-Window Runtime Gate

- invariant:
  Chart Torture pan/zoom diagnostics must prove that the chart's sampled data window changes after
  runtime interaction. Repeated initial prepaint samples or recreated canvas nodes with the same
  sampling key are not enough evidence.
- finding:
  the old suite accepted `total_actions >= 1` and could pass on an initial
  `chart_sampling_window_shift` even when pan/zoom did not change the sampling key. Raising the
  gate exposed `distinct_key_count=1`: the page had added dataZoom specs, but the retained
  `ChartCanvas` could be recreated under the cached Gallery surface and lose widget-local state
  before the post-run bundle.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs`,
  `ecosystem/delinea/src/engine/tests.rs`,
  `crates/fret-diag/src/stats/debug_stats_gates.rs`,
  `crates/fret-diag/src/diag_suite.rs`, and `crates/fret-diag/src/tests.rs`.
- focused delinea gate:
  `cargo nextest run --cargo-profile dev-fast -p delinea interactive_data_zoom_x_pan_and_zoom_updates_output_axis_window --no-fail-fast --no-capture`
  - result: passed; Nextest run id `9424544b-9b5b-4ac8-849b-61d2fd6bd6ec`.
- diagnostics post-run gate unit tests:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag chart_sampling_window_shifts_min build_suite_core_default_post_run_checks_sets_chart_torture_sampling_window_gate --no-fail-fast --no-capture`
  - result: passed; Nextest run id `01f08348-f52b-4423-872d-cf0c3d0f1b00`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  - result: passed; Nextest run id `3acf2b68-8b37-41da-8c73-e25f5820e177`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-chart,gallery-dev`
  - result: passed.
- runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target/fret-diag-chart-torture-suite-shared-engine-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; suite summary
    `target/fret-diag-chart-torture-suite-shared-engine-v2/sessions/1779129994544-98640/suite.summary.json`;
    run id `1779130059382`.
  - sampling-window post-run proof:
    `target/fret-diag-chart-torture-suite-shared-engine-v2/sessions/1779129994544-98640/check.chart_sampling_window_shifts_min.json`
    records `total_actions=3`, `distinct_key_count=2`, and two unique nonzero sampling keys.

## Chart Torture DataZoom Runtime Oracle

- invariant:
  Chart Torture pan/zoom diagnostics must prove a chart-specific interaction state changes, not
  only that screenshot pixels or prepaint sampling keys changed. The shared delinea engine should
  report inactive dataZoom before scripted input and an active X dataZoom window after drag/wheel.
- finding:
  the first output-model oracle failed before interaction because `ChartCanvasOutput` is published
  from `ChartCanvas::paint`. Under ViewCache replay, the app snapshot can see the page and shared
  engine while the output model is still at revision `0`. This was an oracle design issue, not a
  chart runtime defect.
- implementation anchors:
  `apps/fret-ui-gallery/src/harness.rs`,
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs`, and
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-chart-torture-pan-zoom.json`.
- app snapshot proof:
  `target/fret-diag-chart-torture-suite-output-oracle-v2/sessions/1779131582955-88188/1779131647234/bundle.schema2.json`
  records `app_snapshot.chart_torture.x_data_zoom.active=true`,
  `output_model.domain_windows_count=2`, and `output_model.tooltip_lines_count=2` after
  interaction.
- initial failed oracle evidence:
  `target/fret-diag-chart-torture-suite-output-oracle-v1/sessions/1779131062370-101204/script.result.json`
  failed at step 10 waiting for `/chart_torture/x_window/present=true`; the timeout bundle showed
  `output_model.revision=0` throughout the pre-interaction ViewCache replay.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  - result: passed; Nextest run id `b48b7c47-8d4a-4d7d-8923-c3451a4060fe`.
- registry:
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-chart,gallery-dev`
  - result: passed.
- runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target/fret-diag-chart-torture-suite-output-oracle-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; suite summary
    `target/fret-diag-chart-torture-suite-output-oracle-v2/sessions/1779131582955-88188/suite.summary.json`;
    run id `1779131647234`.
  - sampling-window post-run companion:
    `target/fret-diag-chart-torture-suite-output-oracle-v2/sessions/1779131582955-88188/check.chart_sampling_window_shifts_min.json`
    records `total_actions=3`, `distinct_key_count=2`, and two unique nonzero sampling keys.

## Chart Torture Tooltip and Axis Output Runtime Oracle

- invariant:
  after scripted Chart Torture pan/zoom, the paint-published chart output must expose a current X
  axis output window and tooltip/axis-pointer text payload. A dataZoom state change alone is not
  enough if the output model remains stale.
- finding:
  no new chart mechanism defect was reproduced. The existing shared-engine Chart Torture path
  publishes the expected output model after interaction: two domain windows and two tooltip lines.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-chart-torture-pan-zoom.json`,
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`, and
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  - result: passed; Nextest run id `2a111ce6-47bf-4c4d-8a1c-4f18abfb29a2`.
- registry:
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target/fret-diag-chart-torture-suite-tooltip-output-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; suite summary
    `target/fret-diag-chart-torture-suite-tooltip-output-v1/sessions/1779132036567-82724/suite.summary.json`;
    run id `1779132056758`.
  - app snapshot proof:
    final snapshots in
    `target/fret-diag-chart-torture-suite-tooltip-output-v1/sessions/1779132036567-82724/1779132056758/bundle.schema2.json`
    record `x_axis_output_window.present=true`, `output_model.domain_windows_count=2`, and
    `output_model.tooltip_lines_count=2`.
  - sampling-window post-run companion:
    `target/fret-diag-chart-torture-suite-tooltip-output-v1/sessions/1779132036567-82724/check.chart_sampling_window_shifts_min.json`
    records `total_actions=3`, `distinct_key_count=2`, and two unique nonzero sampling keys.

## Workspace Shell Tabstrip Overflow Selection Gate

- invariant:
  workspace shell tabstrip overflow must remain keyboard/mouse reachable under constrained window
  widths, and selecting a hidden tab through the overflow menu must update the active tab mirror and
  UI Gallery selected page. Keyboard tab commands handled by the workspace command scope must use
  the same visible-order policy as the Gallery driver fallback.
- finding:
  the new overflow script passed once the test window was widened from `420 x 720` to `900 x 720`;
  the narrower width collapsed the top-bar center/tabstrip area to zero, so no overflow button was
  reachable. The full workspace shell suite then exposed a real UI Gallery policy drift:
  `workspace.tab.next` handled by `WorkspaceCommandScope` used the default MRU cycle and returned
  from Field to Overlay, while the Gallery driver fallback and command smoke expected Field to
  advance to Command in visible order.
- implementation anchors:
  `ecosystem/fret-workspace/src/tabs.rs`,
  `apps/fret-ui-gallery/src/driver/workspace_nav.rs`,
  `apps/fret-ui-gallery/src/driver/render_flow.rs`,
  `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-tab-commands-smoke.json`,
  `tools/diag-scripts/ui-gallery/workspace-tabstrip/ui-gallery-workspace-tabstrip-overflow-select-command.json`,
  `tools/diag-scripts/suites/ui-gallery-workspace-shell/suite.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- failed-suite evidence before the policy fix:
  `target/fret-diag-workspace-tabstrip-overflow-select-command-v3/sessions/1779133658058-86264/suite.summary.json`
  and
  `target/fret-diag-workspace-shell-tab-commands-smoke-single-v2/sessions/1779134778213-66612/1779134873251-script-step-0016-wait_until-timeout/bundle.schema2.json`
  showed `workspace.tab.next handled=true handled_by_scope=widget` while
  `app_snapshot.selected_page == "overlay"` and the active tab remained Overlay.
- focused gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery workspace_layout_tab_next_uses_gallery_visible_order --no-fail-fast --no-capture`
  - result: passed; Nextest run id `c7040c5e-c9cd-4bdc-a4f4-62fc939cffd2`.
  `cargo nextest run --cargo-profile dev-fast -p fret-workspace tabs::tests::mru_next_toggles_between_two_most_recent --no-fail-fast --no-capture`
  - result: passed; Nextest run id `f11787fb-15b0-488b-951c-2fc272c9f5ee`; this keeps the
    workspace crate default MRU behavior locked for editor-style consumers.
- protocol and registry gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_workspace_tabstrip_overflow_select_command --no-fail-fast --no-capture`
  - result: passed; Nextest run id `9d9ea3d1-66b3-4db3-9a57-cde718d027b0`.
  `python tools/test_check_diag_scripts_registry.py`
  - result: passed; 39 tests.
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-tab-commands-smoke.json --dir target/fret-diag-workspace-shell-tab-commands-smoke-after-inorder-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779136113786`; AI packet
    `target/fret-diag-workspace-shell-tab-commands-smoke-after-inorder-v1/sessions/1779136037789-106088/1779136113786/ai.packet`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-workspace-shell --dir target/fret-diag-workspace-shell-suite-after-overflow-inorder-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; suite summary
    `target/fret-diag-workspace-shell-suite-after-overflow-inorder-v1/sessions/1779136163498-100640/suite.summary.json`.
  - script run ids:
    chrome screenshot `1779136173632`, focus command scope `1779136260333`, tab commands smoke
    `1779136295210`, and tabstrip overflow select command `1779136342864`.

## Workspace Shell Demo Tab Movement Ownership Gate

- invariant:
  workspace shell tab drag/drop must move the tab that started the drag, not whichever tab is active
  by the time command dispatch reaches the app model. End-drop reorders must resolve to a concrete
  tab target, local drag state must survive tabstrip subtree rebuilds, and overflow scripts must
  start drags from visible hit-testable tab bounds.
- finding:
  the runtime reorder mechanism needed ownership hardening: tab strip intents still used
  active-tab reorder commands, so focus/activation changes during drag could make the wrong tab the
  subject of the move. The end-drop target was also a symbolic `End` target until late in the
  path. After adding specific-tab move commands and resolving end-drop to a concrete after-target,
  the overflow-reorder failure was a script authoring defect: the script dragged from a clipped
  `doc-a-0` semantic bounds edge instead of first making that tab visible. A later rebuilt-suite
  failure exposed a separate app-shell ownership defect: the demo runner applied the same
  `workspace.*` command to the app model that `WorkspaceCommandScope` replayed afterward, so
  non-idempotent commands such as `workspace.tab.toggle_pin` could run twice.
- implementation anchors:
  `ecosystem/fret-workspace/src/commands.rs`,
  `ecosystem/fret-workspace/src/tabs.rs`,
  `ecosystem/fret-workspace/src/tab_strip/intent.rs`,
  `ecosystem/fret-workspace/src/tab_strip/mod.rs`,
  `ecosystem/fret-workspace/src/tab_strip/drag_state.rs`,
  `ecosystem/fret-workspace/src/command_scope.rs`,
  `ecosystem/fret-workspace/tests/workspace_command_scope_focus_tab_strip_from_outside_pane.rs`,
  `ecosystem/fret-workspace/src/panes.rs`,
  `apps/fret-examples/src/workspace_shell_demo.rs`,
  `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-activate-hidden-smoke.json`,
  `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-overflow-smoke.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- focused gates:
  `cargo test --profile dev-fast -p fret-workspace --lib end_drop_release_resolves_to_specific_after_target -- --nocapture`
  - result: passed.
  `cargo test --profile dev-fast -p fret-workspace --lib move_specific_tab_before_after_does_not_depend_on_active_tab -- --nocapture`
  - result: passed.
  `cargo test --profile dev-fast -p fret-workspace --lib move_specific_tab_commands_do_not_cross_pinned_boundary -- --nocapture`
  - result: passed.
  `cargo test --profile dev-fast -p fret-workspace --test workspace_command_scope_focus_tab_strip_from_outside_pane -- --nocapture`
  - result: passed; proves `apply_workspace_model_commands(false)` suppresses generic model
    command replay while preserving focus-transfer hooks.
- protocol and registry gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_cross_pane_move_to_end script_v2_roundtrip_workspace_shell_demo_tab_overflow_activate_hidden_smoke --no-fail-fast --no-capture`
  - result: passed; Nextest run id `53b23aaa-eca2-43aa-8e4b-201a0ed6f152`.
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/workspace-shell-demo-tab-pin-commits-preview-smoke.json --dir target/fret-diag-workspace-shell-demo-pin-preview-after-scope-owner-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target/dev-fast/workspace_shell_demo.exe`
  - result: passed; run id `1779147052955`; AI packet
    `target/fret-diag-workspace-shell-demo-pin-preview-after-scope-owner-v1/sessions/1779147049852-103016/1779147052955/ai.packet`.
  `target/dev-fast/fretboard-dev.exe diag suite workspace-shell-demo --dir target/fret-diag-workspace-shell-demo-suite-after-scope-owner-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/workspace_shell_demo.exe`
  - result: passed; suite summary
    `target/fret-diag-workspace-shell-demo-suite-after-scope-owner-v1/sessions/1779147074217-22776/suite.summary.json`.
  - script run ids:
    cross-pane end-drop `1779147077239`, drag-and-scroll `1779147097134`,
    drag-to-split-right `1779147109480`, overflow activate hidden `1779147140088`,
    pin preview `1779147148834`, pinned boundary toggle `1779147156768`,
    pinned cross-boundary drop `1779147168117`, preview commit keeps old tab `1779147179801`,
    preview replaces existing `1779147187987`, and overflow reorder `1779147195864`.
- static checks:
  `rustfmt --edition 2024 --check apps/fret-examples/src/workspace_shell_demo.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs ecosystem/fret-workspace/src/command_scope.rs ecosystem/fret-workspace/src/commands.rs ecosystem/fret-workspace/src/panes.rs ecosystem/fret-workspace/src/tab_strip/drag_state.rs ecosystem/fret-workspace/src/tab_strip/intent.rs ecosystem/fret-workspace/src/tab_strip/mod.rs ecosystem/fret-workspace/src/tabs.rs`
  - result: passed.
  `git diff --check`
  - result: passed.

## Workspace Shell Demo Dirty Close Button Gate

- invariant:
  dirty-close policy must apply when the close request originates from the real tab close button,
  not only from the demo debug close-active command path. A close-by-id widget command for a dirty
  tab must dispatch as handled, show the dirty-close prompt, preserve the tab on Cancel, and remove
  the tab only after Discard.
- finding:
  the first focused runtime run exposed an app-shell redraw gap. `handle_command` installed the
  dirty-close prompt model through `outcome.blocked_dirty_close`, but the generic redraw condition
  only requested redraw for applied outcomes or UI-driver fallback dispatch. The prompt therefore
  existed in app state but did not render. The demo now requests redraw when a dirty-close request
  is blocked.
- diagnostics authoring note:
  the modal prompt barrier filters background tab selectors while the prompt is open. The script
  verifies preservation by clicking Cancel first, waiting for the prompt to disappear, and then
  asserting the tab and dirty marker still exist.
- implementation anchors:
  `apps/fret-examples/src/workspace_shell_demo.rs`,
  `ecosystem/fret-workspace/src/tabs.rs`,
  `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-dirty-shows-prompt-smoke.json`,
  `tools/diag-scripts/workspace-shell-demo-tab-close-button-dirty-shows-prompt-smoke.json`,
  `tools/diag-scripts/suites/workspace-shell-demo/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-button-dirty-shows-prompt-smoke.json > $null`
  - result: passed.
- focused workspace mechanism gate:
  `cargo test --profile dev-fast -p fret-workspace --lib dirty_close_policy_can_block_close_by_id -- --nocapture`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_button_dirty_shows_prompt_smoke --no-fail-fast --no-capture`
  - result: passed; Nextest run id `4c8b4510-ec3f-421a-b0dc-826a0faa27ed`.
- registry:
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- formatting:
  `rustfmt --edition 2024 --check apps\fret-examples\src\workspace_shell_demo.rs ecosystem\fret-workspace\src\tabs.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- build:
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`
  - result: passed.
- static diff check:
  `git diff --check`
  - result: passed.

## AI FileTree Protocol Coverage And Auto-Height VirtualList Refresh

- invariant:
  promoted AI FileTree scripts must survive diagnostics protocol roundtrip, and an auto-height
  `VirtualList` measured leaf must remeasure the parent flow whenever layout-affecting state changes
  row count or items revision. Semantics rows and hit-test geometry must stay inside the same
  expanded FileTree envelope.
- finding:
  the fresh `ui-gallery-ai-file-tree` runtime suite reproduced the stale measured-leaf failure
  shape: `ui-ai-file-tree-file-lib` existed in semantics, but the click point hit the following
  Basic Usage docs section because parent flow reused the old `VirtualList` intrinsic height. The
  focused Rust regression failed with list height `30` instead of `60` before the fix. The screenshot
  script also had two authoring weaknesses: it used a fixed two-frame wait after expanding `src`,
  and it waited for a hidden marker with ordinary `exists` instead of a raw hidden-semantics
  predicate.
- implementation anchors:
  `crates/fret-ui/src/layout/engine/flow.rs`,
  `crates/fret-ui/src/declarative/tests/virtual_list/measurement.rs`,
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-file-tree-demo-screenshot-zinc-dark.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON and registry:
  `python -m json.tool tools\diag-scripts\ui-gallery\ai\ui-gallery-ai-file-tree-demo-screenshot-zinc-dark.json > $null`
  - result: passed.
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-ui\src\layout\engine\flow.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- focused regression gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui auto_height_virtual_list_len_growth_reflows_following_siblings --no-fail-fast --no-capture`
  - result: passed; Nextest run id `de3f626b-824f-4d21-82af-251d51680c64`.
- VirtualList family gate:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui virtual_list --no-fail-fast --no-capture`
  - result: passed; 50/50 tests; Nextest run id `a2e88f71-2c4c-431d-9b0c-8cefdced2a4b`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_ai_file_tree --no-fail-fast --no-capture`
  - result: passed; 4/4 tests; Nextest run id `ea3bdd56-e255-4d34-97f4-b97599cb7369`.
- focused screenshot diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery-ai-file-tree-demo-screenshot-zinc-dark.json --dir target\fret-diag-ai-file-tree-screenshot-zinc-dark-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779168079984`; AI packet
    `target/fret-diag-ai-file-tree-screenshot-zinc-dark-v2/sessions/1779168068402-29976/1779168079984/ai.packet`.
- full runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-ai-file-tree --dir target\fret-diag-ai-file-tree-suite-v3 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; 4/4 scripts; suite summary
    `target/fret-diag-ai-file-tree-suite-v3/sessions/1779168118270-70184/suite.summary.json`;
    screenshot script run id `1779168265307`; `scripts_with_evidence=4`.
- static diff check:
  `git diff --check`
  - result: passed.

Next slice recommendation:

- AI FileTree now has focused mechanism coverage, protocol coverage, strict hidden-marker semantics
  coverage, and fresh runtime suite evidence. Continue to another auto-size measured-leaf runtime
  surface only if it can expose stale parent flow, overlap, or hit-test drift outside FileTree.

- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace-shell-demo-tab-close-button-dirty-shows-prompt-smoke.json --dir target\fret-diag-workspace-shell-demo-dirty-close-widget-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; run id `1779148945096`; AI packet
    `target/fret-diag-workspace-shell-demo-dirty-close-widget-v3/sessions/1779148942029-109108/1779148945096/ai.packet`.
- full runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-dirty-close-widget-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; 11/11 scripts passed; suite summary
    `target/fret-diag-workspace-shell-demo-suite-dirty-close-widget-v1/sessions/1779148963907-13484/suite.summary.json`;
    new dirty-close button script run id `1779148967346`.

## Workspace Shell Demo Close Others Dirty Aggregation Gate

- invariant:
  aggregate tab-close commands must build a dirty-close request over the actual target set, not
  just the active tab. `Close Other Tabs` should target non-pinned, non-active tabs in order,
  include only dirty target tabs in the dirty list, block before mutation on Cancel, and close the
  target set only after Discard.
- finding:
  no runtime mechanism defect was reproduced. The focused runtime gate passed once the script used
  the existing stable tabstrip keyboard-selection path. The first runtime drafts exposed two script
  authoring issues: direct tab click did not select `doc-a-0` in this shell state, and `arrowright`
  was not a valid `press_key` token; the script now uses `arrow_right`.
- diagnostics surface:
  the workspace shell dirty-close dialog now publishes a stable semantics label with
  `reason`, `active`, `close_count`, and `dirty` fields. This lets diagnostics assert aggregate
  prompt content with `label_contains` instead of relying on child text-node aggregation.
- implementation anchors:
  `apps/fret-examples/src/workspace_shell_demo.rs`,
  `ecosystem/fret-workspace/src/tabs.rs`,
  `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json`,
  `tools/diag-scripts/workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json`,
  `tools/diag-scripts/suites/workspace-shell-demo/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json > $null`
  - result: passed.
  `python -m json.tool tools\diag-scripts\workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json > $null`
  - result: passed.
- registry:
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- focused workspace mechanism gate:
  `cargo test --profile dev-fast -p fret-workspace --lib dirty_close_policy_can_block_close_others_with_multiple_targets -- --nocapture`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_others_dirty_aggregation_smoke --no-fail-fast --no-capture`
  - result: passed; Nextest run id `c5d88a4e-1708-43e1-aac0-39bd3f49db41`.
- formatting:
  `rustfmt --edition 2024 --check apps\fret-examples\src\workspace_shell_demo.rs ecosystem\fret-workspace\src\tabs.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- build:
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace-shell-demo-tab-close-others-dirty-aggregation-smoke.json --dir target\fret-diag-workspace-shell-demo-close-others-dirty-aggregation-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; run id `1779150581545`; AI packet
    `target/fret-diag-workspace-shell-demo-close-others-dirty-aggregation-v3/sessions/1779150577000-104072/1779150581545/ai.packet`.
- full runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-close-others-dirty-aggregation-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; 12/12 scripts passed; suite summary
    `target/fret-diag-workspace-shell-demo-suite-close-others-dirty-aggregation-v1/sessions/1779150610325-113064/suite.summary.json`;
    aggregate dirty-close script run id `1779150627934`.

## Workspace Shell Demo Cross-Pane Close Button Ownership Gate

- invariant:
  close-button commands from a tab in a non-active pane must first establish pane ownership before
  applying the tab-close model command. Otherwise the app-owned `WorkspaceWindowLayout` can route
  the close to the previously active pane instead of the pane that owns the clicked tab.
- finding:
  no runtime mechanism defect was reproduced. The existing `WorkspaceTabStripClosePress` path
  carries the pane-activate command and dispatches `workspace.pane.activate.pane-b` before
  `workspace.tab.close.doc-b-1`, so the app model mutates pane-b, not the previously active pane-a.
- diagnostics surface:
  the script asserts both command dispatch trace entries from the real close-button source
  `workspace-shell-pane-pane-b-tab-doc-b-1.close`, then checks `doc-b-1` disappears, `doc-b-0`
  remains selected with set size `1`, and pane-a's `doc-a-2` remains present.
- implementation anchors:
  `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json`,
  `tools/diag-scripts/workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json`,
  `tools/diag-scripts/suites/workspace-shell-demo/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json > $null`
  - result: passed.
  `python -m json.tool tools\diag-scripts\workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json > $null`
  - result: passed.
- registry:
  `python tools/check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_cross_pane_button_ownership_smoke --no-fail-fast --no-capture`
  - result: passed; Nextest run id `dfb8718a-4d49-4fbe-aacd-05b732b5f971`.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- build:
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-cross-pane-button-ownership-smoke.json --dir target\fret-diag-workspace-shell-demo-cross-pane-close-ownership-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; run id `1779151906508`; AI packet
    `target/fret-diag-workspace-shell-demo-cross-pane-close-ownership-v1/sessions/1779151900949-103552/1779151906508/ai.packet`.
- full runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-cross-pane-close-ownership-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; 13/13 scripts passed; suite summary
    `target/fret-diag-workspace-shell-demo-suite-cross-pane-close-ownership-v1/sessions/1779152081871-77896/suite.summary.json`;
    cross-pane close ownership script run id `1779152100416`.

## Workspace Shell Demo Cross-Pane Close Others Context-Menu Ownership Gate

- invariant:
  context-menu aggregate commands from a tab in a non-active pane must establish pane ownership
  before applying the aggregate model command. Otherwise `workspace.tab.close.others` can close
  tabs relative to the previously active pane rather than the pane that owns the context-clicked
  tab.
- finding:
  no runtime ownership defect was reproduced. The real right-click path activates pane-b before the
  `Close Other Tabs` item dispatches `workspace.tab.close.others`, so the app model closes only
  pane-b's other tab and leaves pane-a intact.
- diagnostics surface:
  the script asserts handled `workspace.pane.activate.pane-b`, asserts the aggregate close command
  is pointer-sourced from `workspace-shell-pane-pane-b-tab-doc-b-1.menu.close_others`, then checks
  `doc-b-0` disappears, `doc-b-1` remains selected with set size `1`, and pane-a tabs plus selected
  `doc-a-2` remain present.
- diagnostics attribution note:
  the first focused draft showed `workspace.pane.activate.pane-b` recorded as
  `source_kind=programmatic`, `source_test_id=None`, and `handled_by_driver=true` even though it was
  triggered by the right-click. This is a source-attribution weakness in diagnostics rather than an
  ownership defect. The follow-up source-attribution gate below now closes this gap.
- implementation anchors:
  `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json`,
  `tools/diag-scripts/workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json`,
  `tools/diag-scripts/suites/workspace-shell-demo/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json > $null`
  - result: passed.
  `python -m json.tool tools\diag-scripts\workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json > $null`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_others_cross_pane_context_menu_ownership_smoke --no-fail-fast --no-capture`
  - result: passed; Nextest run id `e7ce6c13-3096-4fb6-a9f1-7a5c81409066`.
- build:
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`
  - result: passed.
- static diff check:
  `git diff --check`
  - result: passed.

- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json --dir target\fret-diag-workspace-shell-demo-cross-pane-context-close-others-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; run id `1779152893863`; AI packet
    `target/fret-diag-workspace-shell-demo-cross-pane-context-close-others-v2/sessions/1779152888206-118016/1779152893863/ai.packet`.
- full runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-cross-pane-context-close-others-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; 14/14 scripts passed; suite summary
    `target/fret-diag-workspace-shell-demo-suite-cross-pane-context-close-others-v1/sessions/1779153282522-114068/suite.summary.json`;
    context-menu Close Others ownership script run id `1779153324733`.

## Workspace Shell Demo Right-Click Pane Activation Source Attribution

- invariant:
  pane activation dispatched by a pointer-down path should carry pointer-source diagnostics, even
  when the activation is emitted by the pane-level policy hook before inner tab/context-menu hooks
  run.
- finding:
  the F205 attribution gap was real diagnostics behavior. The pane pointer region dispatched
  `workspace.pane.activate.pane-b` without first recording pending source metadata, so the demo
  runner consumed `programmatic` even though the trigger was a right-click on
  `workspace-shell-pane-pane-b-tab-doc-b-1`.
- fix:
  `ecosystem/fret-workspace/src/panes.rs` now records pending command dispatch source for the pane
  activation command. It uses `PointerDownCx.hit_pressable_target` when available, so the source
  points at the clicked tab pressable instead of the pane container.
- diagnostics surface:
  `workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json` now asserts
  `workspace.pane.activate.pane-b` with `source_kind=pointer` and
  `source_test_id=workspace-shell-pane-pane-b-tab-doc-b-1`, then keeps the existing menu-item
  pointer-source assertion for `workspace.tab.close.others`.
- implementation anchors:
  `ecosystem/fret-workspace/src/panes.rs`,
  `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`, and
  `tools/diag-scripts/suites/workspace-shell-demo/suite.json`.
- formatting:
  `rustfmt --edition 2024 --check ecosystem\fret-workspace\src\panes.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_tab_close_others_cross_pane_context_menu_ownership_smoke --no-fail-fast --no-capture`
  - result: passed; Nextest run id `bb71150c-c340-4217-9dee-e71eaab872f9`.
- workspace tests:
  `cargo nextest run --cargo-profile dev-fast -p fret-workspace --lib --no-fail-fast`
  - result: passed; 72/72 tests passed; Nextest run id
    `8e889da8-a462-49bf-9685-1bb9750deba6`.
- build:
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`
  - result: passed.
- static diff check:
  `git diff --check`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-tab-close-others-cross-pane-context-menu-ownership-smoke.json --dir target\fret-diag-workspace-shell-demo-cross-pane-context-close-others-source-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; run id `1779156237310`; AI packet
    `target/fret-diag-workspace-shell-demo-cross-pane-context-close-others-source-v1/sessions/1779156234065-53332/1779156237310/ai.packet`.
- full runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-cross-pane-context-source-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; 14/14 scripts passed; suite summary
    `target/fret-diag-workspace-shell-demo-suite-cross-pane-context-source-v1/sessions/1779156335684-11164/suite.summary.json`;
    strengthened context-menu Close Others ownership script run id `1779156370933`.

## Workspace Shell Demo Window-Close Dirty Aggregation Gate

- invariant:
  window-level close requests must aggregate dirty tabs across all panes and block through the
  workspace dirty-close policy before the app closes the window. The real `window.close` command
  and OS close request path must not bypass tab close policy.
- finding:
  a real app-shell defect was confirmed. The workspace shell demo's `window.close` command and
  `Event::WindowCloseRequested` path pushed `WindowRequest::Close` directly, so dirty tabs in any
  pane could be lost without the dirty-close prompt. The fix lives in the owning workspace/app
  policy layer: `fret-workspace` now builds a `CloseWindow` dirty-close request across panes, and
  the demo routes both command and event paths through that policy.
- diagnostics surface:
  `workspace-shell-demo-window-close-dirty-aggregation-smoke.json` marks `doc-a-2` dirty in
  pane-a and `doc-b-1` dirty in pane-b, clicks `workspace-shell-debug-close-window`, waits for a
  pointer-sourced `window.close` command dispatch, asserts the prompt label contains
  `reason=CloseWindow active=doc-a-2 close_count=5` and `dirty=[doc-a-2, doc-b-1]`, cancels, and
  proves both pane roots and dirty markers remain.
- diagnostics authoring note:
  the first focused drafts exposed harness gaps while making the gate strict. The `window.close`
  driver branch did not record a command-dispatch trace, the shared debug command button dispatched
  without pending pointer source metadata, and the suite redirect file used the wrong v1 shape.
  These are fixed by the demo command-dispatch helpers and the `kind=script_redirect` redirect.
- implementation anchors:
  `ecosystem/fret-workspace/src/close_policy.rs`,
  `ecosystem/fret-workspace/src/layout.rs`,
  `apps/fret-examples/src/workspace_shell_demo.rs`,
  `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-window-close-dirty-aggregation-smoke.json`,
  `tools/diag-scripts/workspace-shell-demo-window-close-dirty-aggregation-smoke.json`,
  `tools/diag-scripts/suites/workspace-shell-demo/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-window-close-dirty-aggregation-smoke.json > $null`
  - result: passed.
  `python -m json.tool tools\diag-scripts\workspace-shell-demo-window-close-dirty-aggregation-smoke.json > $null`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- formatting:
  `rustfmt --edition 2024 --check ecosystem\fret-workspace\src\close_policy.rs ecosystem\fret-workspace\src\layout.rs apps\fret-examples\src\workspace_shell_demo.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- focused workspace policy:
  `cargo nextest run --cargo-profile dev-fast -p fret-workspace window_close_dirty_policy_aggregates_tabs_across_panes --no-fail-fast --no-capture`
  - result: passed; Nextest run id `8cf5ad37-5f56-4572-bab9-b3d96f5a29ae`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_workspace_shell_demo_window_close_dirty_aggregation_smoke --no-fail-fast --no-capture`
  - result: passed; Nextest run id `a076a73a-6fe1-44c5-9757-1fd257a67a0c`.
- build:
  `cargo build --profile dev-fast -p fret-demo --bin workspace_shell_demo`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\workspace\shell-demo\workspace-shell-demo-window-close-dirty-aggregation-smoke.json --dir target\fret-diag-workspace-shell-demo-window-close-dirty-aggregation-v4 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; run id `1779171091877`; AI packet
    `target/fret-diag-workspace-shell-demo-window-close-dirty-aggregation-v4/sessions/1779171088566-57484/1779171091877/ai.packet`.
- full runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite workspace-shell-demo --dir target\fret-diag-workspace-shell-demo-suite-window-close-dirty-aggregation-v2 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\workspace_shell_demo.exe`
  - result: passed; suite summary
    `target/fret-diag-workspace-shell-demo-suite-window-close-dirty-aggregation-v2/sessions/1779171327648-11792/suite.summary.json`;
    window-close dirty aggregation script run id `1779171369594`.
- static diff check:
  `git diff --check`
  - result: passed.

## Retained DataTable Column Actions And Stale Script Gates

- invariant:
  retained DataTable column menu actions must dispatch from the real pointer source and keep the
  visible table, retained summaries, and Columns menu state coherent after a column is pinned,
  sorted, and hidden.
- finding:
  no retained DataTable UI/model stale-state defect was reproduced. The strengthened column-actions
  runtime gate passed once the script asserted command dispatch and post-hide retained summaries.
  The broader suite exposed diagnostics authoring drift instead: old toolbar scripts used
  unscoped ids after the torture toolbar adopted `ui-gallery-data-table-torture-toolbar-*`, and
  the old window-boundary script could stall on `wait_frames` after wheel input when the app was
  otherwise idle.
- diagnostics surface:
  `ui-gallery-data-table-retained-column-actions-menu.json` now asserts pointer-sourced command
  dispatch for `pin_left`, `sort_asc`, and `hide` on `mem_mb`; verifies `mem_mb` is hidden from
  the table; verifies sorting and pinning summaries persist; and verifies the Columns menu reports
  `mem_mb` unchecked. The window-boundary gate now asserts row-window movement directly instead of
  waiting for fixed frames.
- diagnostics follow-up note:
  the attempted DataTable window-boundary retained-reconcile counter oracle was not stable on this
  path, even though row 25 appeared and row 0 detached. Keep this gate on observable row-window
  behavior unless a later bundle shows a reliable DataTable retained reconcile diagnostic stream.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-column-actions-menu.json`,
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-faceted-filter.json`,
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-reset-filters.json`,
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-faceted-filter-dashed-border-screenshot.json`,
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-window-boundary-scroll-retained.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-window-boundary-scroll-retained.json > $null`
  - result: passed.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_data_table_retained_column_actions_menu script_v2_roundtrip_ui_gallery_data_table_retained_faceted_filter script_v2_roundtrip_ui_gallery_data_table_retained_reset_filters script_v2_roundtrip_ui_gallery_data_table_retained_window_boundary_scroll --no-fail-fast --no-capture`
  - result: passed; Nextest run id `96dcbf8b-13fa-48da-bae6-c930fad77b04`.
- focused column-actions runtime diagnostics:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-column-actions-menu.json --dir target\fret-diag-data-table-retained-column-actions-menu-state-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779157628043`; AI packet
    `target/fret-diag-data-table-retained-column-actions-menu-state-v2/sessions/1779157546485-30336/1779157628043/ai.packet`.
- focused reset/faceted selector diagnostics:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-reset-filters.json --dir target\fret-diag-data-table-retained-reset-filters-scoped-ids-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779158123014`; AI packet
    `target/fret-diag-data-table-retained-reset-filters-scoped-ids-v1/sessions/1779158112692-50208/1779158123014/ai.packet`.
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-faceted-filter.json --dir target\fret-diag-data-table-retained-faceted-filter-scoped-ids-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779158182462`; AI packet
    `target/fret-diag-data-table-retained-faceted-filter-scoped-ids-v1/sessions/1779158171894-54300/1779158182462/ai.packet`.
- focused window-boundary runtime diagnostics:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-window-boundary-scroll-retained.json --dir target\fret-diag-data-table-window-boundary-scroll-retained-deterministic-v5 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779160262364`; AI packet
    `target/fret-diag-data-table-window-boundary-scroll-retained-deterministic-v5/sessions/1779160251641-54688/1779160262364/ai.packet`.
- full retained DataTable suite:
  `$env:FRET_UI_GALLERY_DATA_TABLE_RETAINED='1'; target\dev-fast\fretboard-dev.exe diag suite ui-gallery-data-table-retained --dir target\fret-diag-data-table-retained-suite-column-actions-state-v3 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; 12/12 scripts passed; suite summary
    `target/fret-diag-data-table-retained-suite-column-actions-state-v3/sessions/1779160314350-36592/suite.summary.json`;
    strengthened column-actions script run id `1779160434776`; window-boundary script run id
    `1779160736205`.

## DataTable View-Cache Filter-Shrink Inputs-Change Gate

- invariant:
  the DataTable view-cache torture gate must actually run with UI Gallery view-cache enabled before
  asserting the non-retained virtual-list `inputs_change` invalidation detail. Otherwise a caller
  can accidentally run the script in default mode and weaken the meaning of the runtime pass.
- finding:
  no new mechanism or DataTable recipe defect was reproduced. The existing
  `non_retained_rerender` and `scroll_handle_inputs_change_window_update` oracle still passes; the
  gate now proves its own launch precondition through `env_defaults` and an app snapshot assertion.
- diagnostics surface:
  `ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json` now declares
  `required_launch_features=["gallery-dev"]`, injects `FRET_UI_GALLERY_VIEW_CACHE=1` through
  `env_defaults`, waits for `/view_cache/enabled=true`, then applies the global filter and asserts
  the layout-sourced virtual-list window record with `reason=inputs_change`,
  `apply_mode=non_retained_rerender`, and
  `invalidation_detail=scroll_handle_inputs_change_window_update`.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json`,
  `tools/diag-scripts/suites/ui-gallery-data-table-view-cache-torture/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json > $null`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_data_table_view_cache_filter_shrink_vlist_inputs_change --no-fail-fast --no-capture`
  - result: passed; Nextest run id `19530940-8e8e-477e-9b3e-80f8f0190843`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change.json --dir target\fret-diag-data-table-view-cache-filter-shrink-env-default-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed without manually setting `FRET_UI_GALLERY_VIEW_CACHE`; run id `1779161694881`;
    AI packet
    `target/fret-diag-data-table-view-cache-filter-shrink-env-default-v1/sessions/1779161683796-54152/1779161694881/ai.packet`.
- suite runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target\fret-diag-data-table-view-cache-suite-env-default-v1 --session-auto --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed without manually setting `FRET_UI_GALLERY_VIEW_CACHE`; 1/1 scripts passed;
    suite summary
    `target/fret-diag-data-table-view-cache-suite-env-default-v1/sessions/1779161746388-13892/suite.summary.json`;
    script run id `1779161756820`.
- static diff check:
  `git diff --check`
  - result: passed.

## UI Gallery View Cache Model-Mutation Gate

- invariant:
  the View Cache harness page must preserve model mutation and controlled overlay state through a
  cached subtree. Counter mutation and Popover open/close state should be observable through the
  dedicated `/view_cache` app snapshot payload, not inferred from text or screenshots.
- finding:
  no new view-cache mechanism defect was reproduced. The existing runtime gate still passes and now
  has direct protocol roundtrip coverage, so schema drift in this promoted script is caught before
  runtime.
- diagnostics surface:
  `ui-gallery-view-cache-model-mutation-through-cache.json` injects
  `FRET_UI_GALLERY_START_PAGE=view_cache`, `FRET_UI_GALLERY_VIEW_CACHE=1`, and
  `FRET_UI_GALLERY_VIEW_CACHE_INNER=1`, asserts `/view_cache/enabled=true` and
  `/view_cache/inner_enabled=true`, resets and bumps the cached counter, then opens and closes the
  controlled Popover while asserting `/view_cache/counter` and `/view_cache/popover_open`.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-model-mutation-through-cache.json`,
  `tools/diag-scripts/suites/ui-gallery-view-cache/suite.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\view-cache\ui-gallery-view-cache-model-mutation-through-cache.json > $null`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_view_cache_model_mutation_through_cache --no-fail-fast --no-capture`
  - result: passed; Nextest run id `e96cc371-57d7-46ca-859b-9120a0907d6d`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\view-cache\ui-gallery-view-cache-model-mutation-through-cache.json --dir target\fret-diag-view-cache-model-mutation-roundtrip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779162384646`; AI packet
    `target/fret-diag-view-cache-model-mutation-roundtrip-v1/sessions/1779162372113-24280/1779162384646/ai.packet`.
- suite runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-view-cache --dir target\fret-diag-view-cache-suite-roundtrip-v1 --session-auto --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; 1/1 scripts passed; suite summary
    `target/fret-diag-view-cache-suite-roundtrip-v1/sessions/1779162428017-56424/suite.summary.json`;
    script run id `1779162437682`.
- static diff check:
  `git diff --check`
  - result: passed.

## Resizable Moving Cached Combobox Root-Boundary Protocol Gate

- invariant:
  a cached Combobox source that moves between Resizable panel viewport roots must keep hit-test
  routing, anchored overlay placement, boundary containment, and relation edges coherent, and the
  promoted runtime script must survive diagnostics schema roundtrips.
- finding:
  no new cached movement/root-boundary mechanism defect was reproduced. The existing runtime gate
  still proves the moved source opens after ViewCache reuse, flips to the top with shadcn
  `sideOffset=6`, remains within the right panel/window boundary, and preserves Combobox
  input/listbox relation edges.
- diagnostics surface:
  `ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json` injects
  `FRET_UI_GALLERY_START_PAGE=resizable`,
  `FRET_UI_GALLERY_START_SECTION=ui-gallery-resizable-view-cache-moving-combobox-docsec`,
  `FRET_UI_GALLERY_RESIZABLE_MOVING_CACHED_COMBOBOX=1`, and
  `FRET_UI_GALLERY_VIEW_CACHE=1`; moves the source from left to right; opens the Combobox; waits
  for the anchored-panel placement trace; then captures layout, screenshot, and bundle evidence.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json`,
  `tools/diag-scripts/suites/ui-gallery-resizable/suite.json`,
  `apps/fret-ui-gallery/src/ui/snippets/resizable/moving_cached_combobox.rs`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\resizable\ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json > $null`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_resizable_view_cache_moving_combobox_root_boundary --no-fail-fast --no-capture`
  - result: passed; Nextest run id `c0b75f4d-b758-48c7-9aac-db09a7f02595`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\resizable\ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json --dir target\fret-diag-resizable-view-cache-moving-combobox-roundtrip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779163064132`; AI packet
    `target/fret-diag-resizable-view-cache-moving-combobox-roundtrip-v1/sessions/1779163052541-37388/1779163064132/ai.packet`.
- suite runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-resizable --dir target\fret-diag-resizable-suite-view-cache-roundtrip-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; 2/2 scripts passed; suite summary
    `target/fret-diag-resizable-suite-view-cache-roundtrip-v1/sessions/1779163144561-38700/suite.summary.json`;
    moving cached Combobox run id `1779163184863`; `scripts_with_evidence=2`;
    `overlay_chosen_side_counts.top=2`.
- static diff check:
  `git diff --check`
  - result: passed.

## Command Retained Active-Descendant Action-State Protocol Gate

- invariant:
  a retained/windowed Command active row must not leave a stale active-descendant relation while the
  row is detached, and when the row reattaches its disabled/invoke semantics must reflect the latest
  model state. The promoted runtime script must also survive diagnostics schema roundtrips.
- finding:
  no new retained relation/action-state mechanism defect was reproduced. The runtime gate still
  proves active-descendant clearing on detach and refreshed `disabled=true` plus `invoke=false` on
  reattach. The first full-suite rerun exposed diagnostics authoring drift in the Command
  long-query script instead: a pre-positioning `scroll_into_view` could stall with
  `timeout.no_frames` when the docs demo was already visible.
- diagnostics surface:
  `ui-gallery-command-retained-active-descendant-action-state.json` injects
  `FRET_UI_GALLERY_START_PAGE=command` and `FRET_UI_GALLERY_STATUS_BAR=1`, resets the retained
  Command demo, scrolls the active row away, asserts `active_item_is_none`, disables the active row,
  scrolls it back, then asserts `active_item_is`, `disabled_is=true`, `semantics_action_is
  invoke=false`, and the status label.
- authoring fix:
  `ui-gallery-command-docs-demo-long-query-text.json` now uses
  `ensure_visible(within_window=true)` for the docs-demo content precondition. The input-level
  `scroll_into_view`, long query injection, font trace, overflow, offset, cursor area, viewport, and
  screenshot/bundle assertions remain unchanged.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-retained-active-descendant-action-state.json`,
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-docs-demo-long-query-text.json`,
  `tools/diag-scripts/suites/ui-gallery-command/suite.json`,
  `apps/fret-ui-gallery/src/ui/snippets/command/retained_active_descendant.rs`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\command\ui-gallery-command-docs-demo-long-query-text.json > $null`
  - result: passed.
  `python -m json.tool tools\diag-scripts\ui-gallery\command\ui-gallery-command-retained-active-descendant-action-state.json > $null`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_command_docs_demo_long_query_text script_v2_roundtrip_ui_gallery_command_retained_active_descendant_action_state --no-fail-fast --no-capture`
  - result: passed; Nextest run id `07836627-15f2-45ec-9209-2915b9d38a3e`.
- focused retained action-state runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\command\ui-gallery-command-retained-active-descendant-action-state.json --dir target\fret-diag-command-retained-active-descendant-action-state-roundtrip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779164006100`; AI packet
    `target/fret-diag-command-retained-active-descendant-action-state-roundtrip-v1/sessions/1779163988388-20728/1779164006100/ai.packet`.
- focused long-query runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\command\ui-gallery-command-docs-demo-long-query-text.json --dir target\fret-diag-command-long-query-ensure-visible-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779164428287`; AI packet
    `target/fret-diag-command-long-query-ensure-visible-v1/sessions/1779164416925-56876/1779164428287/ai.packet`.
- full Command suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-command --dir target\fret-diag-command-suite-retained-action-state-roundtrip-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; 18/18 scripts passed; suite summary
    `target/fret-diag-command-suite-retained-action-state-roundtrip-v2/sessions/1779164457116-49144/suite.summary.json`;
    `scripts_with_evidence=18`; long-query run id `1779164551371`; retained action-state run id
    `1779165106416`.
- static diff check:
  `git diff --check`
  - result: passed.
