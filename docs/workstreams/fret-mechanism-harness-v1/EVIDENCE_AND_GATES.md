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
- Element root-bounds cache owner gates:
  `crates/fret-ui/src/declarative/tests/layout/viewport_roots.rs`
  - proof:
    `element_root_bounds_cache_*` covers fast-path frames, overlay-only frames, retained overlay
    parent pointers, ancestor-only relayout while the viewport registration owner is stable, owner
    relayout without viewport-root registration, nearest nested viewport precedence, same-element
    movement between viewport roots, and view-cache-hit movement.
  - first red command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui element_root_bounds_cache_survives_ancestor_layout_when_viewport_owner_is_stable --no-fail-fast --no-capture`
  - first red result:
    failed before the owner fix: the descendant anchor fell back to the 900x1000 window bounds
    instead of the 336x378 Resizable-style viewport bounds.
  - current focused command:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui element_root_bounds_cache --no-fail-fast --no-capture`
  - current focused result:
    passed, 9/9 tests, with Nextest run id `0aff4730-a191-44fb-8ae5-ba7083497ee6`.
- Resizable cached Combobox relation-bounce runtime gate:
  `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-relation-bounce.json`
  - proof:
    starts on the moving cached Combobox Resizable page, proves the popup relation endpoints are gone
    after close, moves the cached source across panels, then reopens and proves `controls` and
    `labelled_by` relation edges plus top-side panel-root placement.
  - focused protocol gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_resizable_view_cache_moving_combobox_relation_bounce --no-fail-fast`
  - protocol result:
    passed with Nextest run id `453ab3da-32c4-49aa-9f36-e16f77889572`.
  - focused runtime repair proof:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-multi-viewport-combobox-placement.json --dir target/fret-diag-resizable-multi-viewport-combobox-placement-owner-fix-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed with run id `1779465182657`; AI packet:
    `target/fret-diag-resizable-multi-viewport-combobox-placement-owner-fix-v1/sessions/1779465172267-75620/1779465182657/ai.packet`.
  - suite command:
    `target/dev-fast/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-resizable/suite.json --dir target/fret-diag-resizable-suite-owner-fix-v1 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - suite result:
    passed 4/4; relation-bounce row run id `1779465331803`; summary:
    `target/fret-diag-resizable-suite-owner-fix-v1/sessions/1779465226892-77780/suite.summary.json`.
- Resizable cached Combobox Escape focus-restore runtime gate:
  `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-escape-focus-restore.json`
  - proof:
    moves the cached Combobox source across Resizable viewport roots before opening the popup,
    asserts focus enters the popup input, then presses Escape and proves the popup unmounts with
    focus restored to the live moved trigger.
  - focused protocol gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_resizable_view_cache_moving_combobox_escape_focus_restore --no-fail-fast`
  - protocol result:
    passed with Nextest run id `90fead2d-9ef6-4f9f-be6c-682569713906`.
  - focused runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-escape-focus-restore.json --dir target/fret-diag-resizable-moving-combobox-escape-focus-restore-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed with run id `1779466372607`; AI packet:
    `target/fret-diag-resizable-moving-combobox-escape-focus-restore-v1/sessions/1779466364091-64312/1779466372607/ai.packet`.
  - suite command:
    `target/dev-fast/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-resizable/suite.json --dir target/fret-diag-resizable-suite-focus-restore-v1 --session-auto --timeout-ms 1000000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - suite result:
    passed 5/5; focus-restore row run id `1779466506265`; summary:
    `target/fret-diag-resizable-suite-focus-restore-v1/sessions/1779466397357-72132/suite.summary.json`.
- Resizable cached Combobox disabled action-state runtime gate:
  `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-disabled-action-state.json`
  - proof:
    toggles the diagnostics-only `In Review` option to disabled, moves the cached Combobox source
    across Resizable viewport roots, reopens the popup, and proves the moved item exports
    `disabled=true` and `invoke=false` with the updated subtree keyed explicitly on the disabled
    state.
  - focused protocol gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_resizable_view_cache_moving_combobox_disabled_action_state --no-fail-fast`
  - protocol result:
    passed with Nextest run id `1088c679-b0ad-4e52-9a14-6d0c4bd8dc09`.
  - focused runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-combobox-disabled-action-state.json --dir target/fret-diag-resizable-moving-combobox-disabled-action-state-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed with run id `1779467419798`; AI packet:
    `target/fret-diag-resizable-moving-combobox-disabled-action-state-v1/sessions/1779467403958-68712/1779467419798/ai.packet`.
  - suite command:
    `target/dev-fast/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-resizable/suite.json --dir target/fret-diag-resizable-suite-disabled-action-state-v1 --session-auto --timeout-ms 1000000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - suite result:
    passed 6/6; disabled-action-state row run id `1779467737769`; summary:
    `target/fret-diag-resizable-suite-disabled-action-state-v1/sessions/1779467543831-72756/suite.summary.json`.

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
- Virtual-list retained selected/action-state bounce gate:
  `tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-retained-selected-action-state-bounce.json`
  - suite membership:
    `tools/diag-scripts/suites/ui-gallery-vlist-retained-action-state/suite.json`
  - proof:
    drives the dev-only Virtual List Torture page with retained keep-alive plus row-cache enabled,
    selects row 2, boundary-scrolls until row 2 detaches, clears the editing/selection model while
    the row is detached, then bounces back and asserts row 2 reattaches with `selected=false` while
    preserving `semantics_action_is(invoke)=true`.
  - implementation anchors:
    `apps/fret-ui-gallery/src/ui/previews/pages/harness/virtual_list_torture.rs`,
    `tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-retained-selected-action-state-bounce.json`,
    `tools/diag-scripts/suites/ui-gallery-vlist-retained-action-state/suite.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - focused runtime evidence:
    `target/fret-diag-vlist-retained-selected-action-state-bounce-v2/sessions/1779431631089-228876/1779431677188/ai.packet`;
    share pack
    `target/fret-diag-vlist-retained-selected-action-state-bounce-v2/sessions/1779431631089-228876/share/1779431677188.zip`.
  - dedicated suite evidence:
    `target/fret-diag-vlist-retained-action-state-v1/sessions/1779433457108-255908/suite.summary.json`
    passed 1/1 with row run id `1779433613838`.
  - retained boundary suite refresh:
    `target/fret-diag-vlist-window-boundary-retained-selected-action-state-v3/sessions/1779433457127-264980/suite.summary.json`
    passed 2/2 after the action-state row moved out of the minimal boundary suite.
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
  - fresh promotion gate:
    the script is now also promoted into
    `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`, and protocol
    roundtrip coverage is locked by
    `script_v2_roundtrip_ui_gallery_command_palette_disabled_focusable_keyboard_suppression`.
  - fresh focused shadcn gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib cmdk_default_disabled_item_is_skipped_by_active_descendant_navigation cmdk_focusable_disabled_item_can_be_active_descendant_without_keyboard_activation`
  - fresh focused shadcn result:
    passed, 2 tests; run id `bfad983b-8637-449b-b925-bfedd2da209d`.
  - fresh protocol roundtrip gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_command_palette_disabled_focusable_keyboard_suppression script_v2_roundtrip_ui_gallery_command_retained_active_descendant_action_state`
  - fresh protocol roundtrip result:
    passed, 2 tests; run id `35eb12fd-44fa-4db1-90cd-3fac0ac3d211`.
  - fresh runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-palette-disabled-focusable-keyboard-suppression.json --dir target/fret-diag-command-disabled-focusable-keyboard-suppression-runtime-evidence-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - fresh runtime result:
    passed; run id `1779407145527`.
  - fresh runtime evidence:
    `target/fret-diag-command-disabled-focusable-keyboard-suppression-runtime-evidence-v2/sessions/1779407084356-243016/1779407145527/ai.packet`
  - fresh runtime pack:
    `target/fret-diag-command-disabled-focusable-keyboard-suppression-runtime-evidence-v2/sessions/1779407084356-243016/share/1779407145527.zip`
  - row-only suite command:
    `target/dev-fast/fretboard-dev.exe diag suite --glob "tools/diag-scripts/ui-gallery/command/ui-gallery-command-palette-disabled-focusable-keyboard-suppression.json" --dir target/fret-diag-command-disabled-focusable-row-suite-v1 --session-auto --timeout-ms 720000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - row-only suite result:
    passed 1/1; `stage_counts={"passed":1}`.
  - row-only suite evidence:
    `target/fret-diag-command-disabled-focusable-row-suite-v1/sessions/1779408495703-242652/suite.summary.json`
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
- Moving cached Popover outside-press root-boundary gate:
  `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-popover-outside-press.json`
  - invariant:
    a cached Popover source that moves between Resizable panel viewport roots must keep panel-root
    overlay placement and non-modal outside-press dismissal/click-through policy after the move.
  - implementation anchors:
    `apps/fret-ui-gallery/src/ui/snippets/resizable/moving_cached_popover.rs`,
    `apps/fret-ui-gallery/src/ui/pages/resizable.rs`,
    `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-popover-outside-press.json`,
    `tools/diag-scripts/suites/ui-gallery-resizable/suite.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - focused protocol gate:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_resizable_view_cache_moving_popover_outside_press --no-fail-fast`
  - protocol result:
    passed with Nextest run id `0f8d9478-ca11-405d-842a-ffa81738151c`.
  - focused runtime command:
    `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-view-cache-moving-popover-outside-press.json --dir target/fret-diag-resizable-moving-popover-outside-press-v3 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - focused runtime result:
    passed with run id `1779469463236`; AI packet:
    `target/fret-diag-resizable-moving-popover-outside-press-v3/sessions/1779469453540-79064/1779469463236/ai.packet`.
  - focused runtime pack:
    `target/fret-diag-resizable-moving-popover-outside-press-v3/sessions/1779469453540-79064/share/1779469463236.zip`.
  - suite command:
    `target/dev-fast/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-resizable/suite.json --dir target/fret-diag-resizable-suite-popover-outside-press-v1 --session-auto --timeout-ms 1200000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - suite result:
    passed 7/7; Popover row run id `1779469704952`; summary:
    `target/fret-diag-resizable-suite-popover-outside-press-v1/sessions/1779469537437-26788/suite.summary.json`.
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
- Carousel state suite and focus autoplay stop:
  - invariant:
    Carousel state evidence should be split into compact, independently runnable suites. Focus
    entry into a slide is an autoplay stopOnInteraction event even when no timer token happens to
    be active in that render frame.
  - findings:
    the first `ui-gallery-carousel-state` run found a real shadcn Carousel defect. The focus
    stopOnInteraction script moved focus into a nested slide button and watchFocus scrolled to the
    slide, but the status remained `playing=true • stopped_by_interaction=false`. The focus stop
    path incorrectly required `runtime_snapshot.autoplay_token.is_some()`, treating a scheduling
    detail as the interaction oracle.
  - implementation anchors:
    `ecosystem/fret-ui-shadcn/src/carousel.rs`,
    `tools/diag-scripts/suites/ui-gallery-carousel-state/suite.json`,
    `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-events-select-gate.json`,
    `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-events-reinit-gate.json`,
    `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-stop-on-last-snap-gate.json`,
    `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-stop-on-interaction-focus-gate.json`,
    `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-rtl-controls-gate.json`, and
    `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
  - JSON/registry:
    `python -m json.tool tools\diag-scripts\suites\ui-gallery-carousel-state\suite.json > $null`
    and `python tools\check_diag_scripts_registry.py`
    - result: passed.
  - formatting:
    `rustfmt --edition 2024 --check ecosystem\fret-ui-shadcn\src\carousel.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
    - result: passed.
  - focused recipe regression:
    `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --test carousel_autoplay_api_handle carousel_autoplay_stop_on_interaction_stops_after_slide_receives_focus --no-fail-fast --no-capture`
    - result: passed; Nextest run id `7fc50006-1357-4756-86f2-9452c5605aab`.
  - protocol roundtrip:
    `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_carousel_state_gates --no-fail-fast --no-capture`
    - result: passed; Nextest run id `96bee069-29be-4c38-8423-f89b44f5d3fa`.
  - build gate:
    `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
    - result: passed.
  - focused runtime diagnostics:
    `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\carousel\ui-gallery-carousel-plugin-autoplay-stop-on-interaction-focus-gate.json --dir target\fret-diag-carousel-stop-on-focus-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
    - result: passed; run id `1779176381271`; AI packet
      `target/fret-diag-carousel-stop-on-focus-v2/sessions/1779176372434-68188/1779176381271/ai.packet`.
  - full state suite:
    `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-carousel-state --dir target\fret-diag-carousel-state-suite-v2 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
    - result: passed; suite summary
      `target/fret-diag-carousel-state-suite-v2/sessions/1779176492375-78748/suite.summary.json`;
      5/5 rows; `scripts_with_evidence=5`; `focus_mismatch_total=0`; row run ids
      `1779176501302`, `1779176563142`, `1779176638167`, `1779176831378`, and
      `1779176928324`.
  - first failed evidence:
    `target/fret-diag-carousel-state-suite-v1/sessions/1779174619173-69116/1779175342309-script-step-0022-wait_until-timeout/bundle.schema2.json`
    captured the focused nested slide button and the stale `playing=true •
    stopped_by_interaction=false` status before the fix.
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

## Chart Torture Visible Domain-Window Runtime Oracle

- invariant:
  after scripted Chart Torture pan/zoom, the visible X domain used by both the engine axis output
  and the paint-published `ChartCanvasOutput` domain-window payload must match the active dataZoom
  window and must differ from the fixture's initial full X domain. Counting published windows is not
  enough if the published window is stale or still equal to the initial full domain.
- finding:
  no new chart mechanism defect was reproduced. The promoted pan/zoom gate now observes the
  expected convergence: before interaction the X axis output is the full fixture domain
  `1735689600000..1747689540000`, and after interaction both the engine axis output and output-model
  X domain publish `1739283224994..1757471732398`, matching the active dataZoom window.
- implementation anchors:
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-chart-torture-pan-zoom.json`,
  `tools/diag-scripts/ui-gallery-chart-torture-pan-zoom.json`, and
  `tools/diag-scripts/suites/ui-gallery-chart-torture/suite.json`.
- static gates:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\driver\diag_snapshot.rs`
  - result: passed.
  `python -m json.tool tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json > $null`
  - result: passed.
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
  `git diff --check`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  - result: passed; Nextest run id `7bbb707d-390a-4659-b782-1d38ef175e24`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-chart,gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json --dir target\fret-diag-chart-torture-visible-window-oracle-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779173616393`; AI packet
    `target/fret-diag-chart-torture-visible-window-oracle-v1/sessions/1779173543971-68812/1779173616393/ai.packet`.
- runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target\fret-diag-chart-torture-suite-visible-window-oracle-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; suite summary
    `target/fret-diag-chart-torture-suite-visible-window-oracle-v1/sessions/1779173643592-70968/suite.summary.json`;
    run id `1779173655069`.
  - app snapshot proof:
    final snapshots in
    `target/fret-diag-chart-torture-suite-visible-window-oracle-v1/sessions/1779173643592-70968/1779173676553-ui-gallery-chart-torture-pan-zoom-after/bundle.schema2.json`
    record `runtime_oracles.x_axis_output_matches_data_zoom=true`,
    `runtime_oracles.x_output_model_domain_matches_data_zoom=true`,
    `runtime_oracles.x_axis_output_changed_from_full_domain=true`, and
    `runtime_oracles.x_output_model_domain_changed_from_full_domain=true`.

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

## Text Paint Reprepare Layout Repair And Combobox Intro Gate

- invariant:
  auto-height text nodes must not keep stale shorter layout bounds when paint-time text preparation
  observes a narrower width or changed font stack and produces taller wrapped metrics. The next
  frame must repair layout before the taller text can keep overlapping following content.
- finding:
  the user-observed Combobox page screenshot showed the docs intro text overlapping the Popup
  heading until a resize forced layout recovery. The stable runtime script did not reproduce a
  persistent post-wait overlap, but the mechanism path was real: paint-time reprepare could update
  the prepared blob and metrics without scheduling layout when the new metrics outgrew auto-height
  bounds.
- fix:
  `Text`, `StyledText`, and `SelectableText` now call the shared paint repair helper after
  reprepare. The helper is restricted to width/font-stack reparations, auto-height layout, and
  prepared height greater than current bounds by more than `0.5px`; it invalidates the current node
  for layout and requests redraw.
- diagnostics surface:
  `ui-gallery-combobox-popup-doc-intro-non-overlap.json` starts UI Gallery on Combobox/Popup at
  `671x460`, captures layout, screenshot, and bundle evidence, and asserts an `8px` vertical gap
  between the doc intro and Popup title plus an `8px` gap between the title and description.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/paint.rs`,
  `crates/fret-ui/src/declarative/tests/text_cache.rs`,
  `apps/fret-ui-gallery/src/ui/doc_layout.rs`,
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-doc-intro-non-overlap.json`,
  `tools/diag-scripts/ui-gallery-combobox-popup-doc-intro-non-overlap.json`,
  `tools/diag-scripts/suites/ui-gallery-combobox-geometry-placement/suite.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\paint.rs crates\fret-ui\src\declarative\tests\text_cache.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_paint_width_shrink_reinvalidates_layout_when_height_grows --no-fail-fast --no-capture`
  - result: passed; Nextest run id `50e6ec15-0b4f-4340-b689-c10ae58055e2`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_non_overlap script_v2_roundtrip_ui_gallery_combobox_popup_trigger script_v2_roundtrip_ui_gallery_combobox_popup_trigger_bottom_room --no-fail-fast --no-capture`
  - result: passed; Nextest run id `05c76ef5-e683-4a03-a809-d71fc53256ca`.
- build/check:
  `cargo check --profile dev-fast -p fret-ui`
  - result: passed with the existing `current_effective_opacity` dead-code warning in
    `crates\fret-ui\src\elements\runtime.rs`.
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-popup-doc-intro-non-overlap.json --dir target\fret-diag-combobox-popup-doc-intro-overlap-671x460-repair-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779186094473`; AI packet
    `target/fret-diag-combobox-popup-doc-intro-overlap-671x460-repair-v1/sessions/1779186086330-88228/1779186094473/ai.packet`.
- full Combobox geometry placement suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-text-layout-repair-v2 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; suite summary
    `target/fret-diag-combobox-geometry-placement-text-layout-repair-v2/sessions/1779186620899-17592/suite.summary.json`;
    new intro non-overlap script run id `1779186747293`.
- static diff check:
  `git diff --check`
  - result: passed.

## Combobox Checkmark Effective Opacity Gate

- invariant:
  selected and unselected Combobox checkmarks are both present in semantics/layout for stable
  geometry, but only the selected checkmark should paint at effective opacity `1.0`; the unselected
  checkmark should remain at effective opacity `0.0`.
- finding:
  no Combobox recipe defect was reproduced. The first focused runtime run exposed a diagnostics
  harness gap instead: predicate-bearing `assert` steps did not borrow `ElementRuntime` unless the
  step contained a `global_element_id` selector, so the new opacity predicate evaluated false
  before reading the target.
- implementation anchors:
  `crates/fret-ui/src/elements/runtime.rs`,
  `crates/fret-ui/src/declarative/mount.rs`,
  `crates/fret-diag-protocol/src/lib.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`,
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-long-text-geometry.json`,
  and
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-geometry.json`.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-ui\src\elements\runtime.rs crates\fret-ui\src\declarative\mount.rs crates\fret-diag-protocol\src\lib.rs ecosystem\fret-bootstrap\src\ui_diagnostics\predicates.rs ecosystem\fret-bootstrap\src\ui_diagnostics\script_steps_wait.rs ecosystem\fret-bootstrap\src\ui_diagnostics\script_engine.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- JSON validation:
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-long-text-geometry.json > $null`
  and
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-geometry.json > $null`
  - result: passed.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_element_effective_opacity_approx_eq_serializes_and_deserializes script_v2_roundtrip_ui_gallery_combobox_long_text_geometry script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_geometry --no-fail-fast --no-capture`
  - result: passed; Nextest run id `021decf3-5aae-41ac-95f6-ec738542acca`.
- diagnostics runtime gate test:
  `cargo test --profile dev-fast -p fret-bootstrap runtime_gate_keeps_effective_opacity_predicates --features ui-app-driver,diagnostics -- --nocapture`
  - result: passed.
- compile/build:
  `cargo check --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics`
  and
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused LTR runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-long-text-geometry.json --dir target\fret-diag-combobox-long-text-opacity-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779180476346`; AI packet
    `target/fret-diag-combobox-long-text-opacity-v2/sessions/1779180467898-80632/1779180476346/ai.packet`.
- focused RTL runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-geometry.json --dir target\fret-diag-combobox-rtl-long-text-opacity-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779180503756`; AI packet
    `target/fret-diag-combobox-rtl-long-text-opacity-v1/sessions/1779180495343-65848/1779180503756/ai.packet`.
- full Combobox geometry suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-opacity-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; 7/7 rows; suite summary
    `target/fret-diag-combobox-geometry-placement-opacity-v1/sessions/1779180495343-74828/suite.summary.json`;
    `scripts_with_evidence=7`; `overlay_chosen_side_counts.bottom=6`; `overlay_chosen_side_counts.top=5`.
  - first failed bundle for diagnostics root-cause comparison:
    `target/fret-diag-combobox-long-text-opacity-v1/sessions/1779179412289-69288/1779179421684/bundle.schema2.json`.

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

## Resizable Multi-Viewport Select Root-Boundary Gate

- invariant:
  a popper-positioned Select opened inside a Resizable panel viewport root must use the panel root,
  not the OS window, as its placement boundary. When the panel has insufficient space below but the
  window still has room, the listbox should flip to the top and keep Select relation edges coherent.
- finding:
  no Select root-boundary defect was reproduced. The new runtime companion proves Select follows
  the same panel-root ownership invariant already covered for Combobox, broadening overlay family
  coverage inside Resizable roots.
- diagnostics surface:
  `ui-gallery-resizable-multi-viewport-select-placement.json` injects
  `FRET_UI_GALLERY_START_PAGE=resizable`,
  `FRET_UI_GALLERY_START_SECTION=ui-gallery-resizable-multi-viewport-select-docsec`, and
  `FRET_UI_GALLERY_RESIZABLE_MULTI_VIEWPORT_SELECT=1`; opens the Select control near the bottom of
  the right panel; waits for a `placed_rect` trace with `chosen_side=top`, `flipped=true`, and
  `side_offset=6`; asserts listbox window containment and relation edges; selects `Release Ready`;
  then reopens and verifies the selected item state.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/resizable/multi_viewport_select.rs`,
  `apps/fret-ui-gallery/src/ui/pages/resizable.rs`,
  `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-multi-viewport-select-placement.json`,
  `tools/diag-scripts/suites/ui-gallery-resizable/suite.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON/registry:
  `python -m json.tool tools\diag-scripts\ui-gallery\resizable\ui-gallery-resizable-multi-viewport-select-placement.json > $null`
  and `python tools\check_diag_scripts_registry.py`
  - result: passed.
- formatting:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\resizable\multi_viewport_select.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_resizable_multi_viewport_select_placement --no-fail-fast --no-capture`
  - result: passed; latest Nextest run id `7944bf63-93b6-476d-aa9a-7b6b53771d9e`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\resizable\ui-gallery-resizable-multi-viewport-select-placement.json --dir target\fret-diag-resizable-multi-viewport-select-placement-v8 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779193025213`; AI packet
    `target/fret-diag-resizable-multi-viewport-select-placement-v8/sessions/1779193017299-99444/1779193025213/ai.packet`.
- full Resizable suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-resizable --dir target\fret-diag-ui-gallery-resizable-suite-select-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; 3/3 scripts; suite summary
    `target/fret-diag-ui-gallery-resizable-suite-select-v1/sessions/1779193114796-63336/suite.summary.json`;
    Select run id `1779193162410`; `scripts_with_evidence=3`.
- static diff check:
  `git diff --check`
  - result: passed.

## Text Reprepare Repair-Frame Clip And Full Combobox Startup Gate

- invariant:
  when paint-time text preparation discovers that a newly prepared auto-height text blob is taller
  than the stale layout bounds, the framework must both schedule a layout repair and prevent that
  same frame from visibly drawing outside the stale text bounds.
- finding:
  the earlier text repair fixed convergence by invalidating layout and requesting redraw, but the
  user-observed full Combobox page screenshot showed why same-frame paint spill also matters: a
  taller wrapped intro can overlap following content until a later layout or manual resize recovers.
- mechanism change:
  `Text`, `StyledText`, and `SelectableText` now draw under `PushClipRRect`/`PopClip` for the
  repair frame when `maybe_repair_text_layout_after_paint_prepare` schedules layout repair. Normal
  non-repair text paint remains unclipped by this helper.
- runtime surface:
  `ui-gallery-combobox-full-page-startup-intro-non-overlap.json` starts on the full Combobox page
  at `671x460`, captures layout/screenshot/bundle evidence, and asserts `docsec-basic-title.top -
  ui-gallery-doc-page-intro.bottom >= 16px`. It complements the existing
  `ui-gallery-combobox-popup-doc-intro-non-overlap.json`, which starts on the focused Popup
  section.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/paint.rs`,
  `crates/fret-ui/src/declarative/tests/text_cache.rs`,
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-full-page-startup-intro-non-overlap.json`,
  `tools/diag-scripts/suites/ui-gallery-combobox-geometry-placement/suite.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- formatting:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\paint.rs crates\fret-ui\src\declarative\tests\text_cache.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- JSON/registry:
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-full-page-startup-intro-non-overlap.json > $null`,
  `python -m json.tool tools\diag-scripts\ui-gallery-combobox-full-page-startup-intro-non-overlap.json > $null`,
  and `python tools\check_diag_scripts_registry.py`
  - result: passed.
- mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_paint_width_shrink_reinvalidates_layout_when_height_grows --no-fail-fast --no-capture`
  - result: passed; latest Nextest run id `d8184adc-9875-470f-9828-025bc220465e`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_full_page_startup_intro_non_overlap --no-fail-fast --no-capture`
  - result: passed; Nextest run id `4fa52001-9eb6-4102-9bba-033f10b3e2c0`.
- build/check:
  `cargo check --profile dev-fast -p fret-ui`
  and `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed. `cargo check -p fret-ui` still reports the pre-existing
    `current_effective_opacity` dead-code warning.
- focused full-page runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-full-page-startup-intro-non-overlap.json --dir target\fret-diag-combobox-full-page-startup-intro-text-clip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779194385483`; AI packet
    `target/fret-diag-combobox-full-page-startup-intro-text-clip-v1/sessions/1779194373536-98264/1779194385483/ai.packet`.
- focused Popup companion runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-popup-doc-intro-non-overlap.json --dir target\fret-diag-combobox-popup-doc-intro-text-clip-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779194199027`.
- full Combobox geometry placement suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-text-clip-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; suite summary
    `target/fret-diag-combobox-geometry-placement-text-clip-v1/sessions/1779194425260-69272/suite.summary.json`;
    full-page startup run id `1779194524638`.

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

## CommandDialog Basic Overlay Focus Gate

- invariant:
  the Basic `CommandDialog` recipe must install a modal dialog overlay with coherent input/listbox
  relations, keep keyboard active-descendant state on the input, and restore focus to the trigger's
  semantic button when Escape closes the dialog.
- finding:
  no overlay focus defect was reproduced. The first focused runtime draft exposed a diagnostics
  authoring issue: `focus_is` compares the focused semantics node directly, so asserting focus on
  `ui-gallery-command-basic-trigger.chrome` targeted the visual chrome child. The failure bundle
  showed the runtime had restored focus to the outer `role=button` node labelled `Open Menu`.
- diagnostics surface:
  `ui-gallery-command-basic-dialog-overlay-focus.json` starts on the Command Basic section, opens
  the real trigger, asserts dialog and close-button semantics, input/listbox/item presence,
  listbox `labelled_by` relation, input `active_descendant` relation, ArrowDown active-row
  movement, listbox window containment, screenshot/layout/bundle evidence, Escape dismissal, and
  final focus on the `Open Menu` button.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/command/ui-gallery-command-basic-dialog-overlay-focus.json`,
  `tools/diag-scripts/ui-gallery-command-basic-dialog-overlay-focus.json`,
  `tools/diag-scripts/suites/ui-gallery-command/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- JSON/registry/format:
  `python -m json.tool tools\diag-scripts\ui-gallery\command\ui-gallery-command-basic-dialog-overlay-focus.json > $null`,
  `python -m json.tool tools\diag-scripts\ui-gallery-command-basic-dialog-overlay-focus.json > $null`,
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`,
  and `python tools\check_diag_scripts_registry.py`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_command_basic_dialog_overlay_focus --no-fail-fast --no-capture`
  - result: passed; Nextest run id `08a923e7-9ca3-4b3c-bb2f-fe62628193ec`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\command\ui-gallery-command-basic-dialog-overlay-focus.json --dir target\fret-diag-command-basic-dialog-overlay-focus-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779196803631`; AI packet
    `target/fret-diag-command-basic-dialog-overlay-focus-v2/sessions/1779196795872-108048/1779196803631/ai.packet`.
- full Command suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-command --dir target\fret-diag-ui-gallery-command-suite-dialog-overlay-focus-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; suite summary
    `target/fret-diag-ui-gallery-command-suite-dialog-overlay-focus-v1/sessions/1779196833923-91304/suite.summary.json`;
    new CommandDialog script run id `1779196993347`.

## Combobox Popup Short Startup Intro Non-Overlap Gate

- invariant:
  the Combobox Popup docs intro must reserve enough measured vertical space before the Popup
  section title on the cold short-window startup path, before a manual resize can repair any stale
  layout.
- finding:
  the screenshot-derived probe did not reproduce a current overlap after the text repair-frame clip
  fix. It still exposed a useful missing gate: the existing `671x460` Popup and full-page startup
  scripts did not pin the shorter logical startup size implied by the observed `994x466` image on a
  1.5x scale display.
- diagnostics surface:
  `ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json` starts directly on the
  Combobox Popup section with `FRET_UI_GALLERY_MAIN_WINDOW_SIZE=663x311`, captures layout,
  screenshot, and bundle evidence at frame 3/5, then asserts the intro-to-title gap is at least
  `16px` and the title-to-description gap is at least `8px`.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json`,
  `tools/diag-scripts/ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json`,
  `tools/diag-scripts/suites/ui-gallery-combobox-geometry-placement/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused AI packet
  `target/fret-diag-combobox-popup-doc-intro-short-startup-v1/sessions/1779198558655-90216/1779198569025/ai.packet`;
  focused pack
  `target/fret-diag-combobox-popup-doc-intro-short-startup-v1/sessions/1779198558655-90216/share/1779198569025.zip`;
  suite summary
  `target/fret-diag-combobox-geometry-placement-short-startup-v1/sessions/1779198616098-23160/suite.summary.json`.
- JSON/registry/format:
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json > $null`,
  `python -m json.tool tools\diag-scripts\ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json > $null`,
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-combobox-geometry-placement\suite.json > $null`,
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`,
  and `python tools\check_diag_scripts_registry.py`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_short_startup_non_overlap --no-fail-fast --no-capture`
  - result: passed; Nextest run id `e92bb8b8-cf66-47b5-8281-1fa91f73c6b3`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-popup-doc-intro-short-startup-non-overlap.json --dir target\fret-diag-combobox-popup-doc-intro-short-startup-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779198569025`; layout at frame 3 shows
    `docsec-popup-title.top - ui-gallery-doc-page-intro.bottom = 24px`.
- full Combobox geometry placement suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-short-startup-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; 10/10 scripts; suite summary
    `target/fret-diag-combobox-geometry-placement-short-startup-v1/sessions/1779198616098-23160/suite.summary.json`;
    new short-startup run id `1779198792251`.

## AI Transcript Non-Retained Scroll Count Gate

- invariant:
  the AI transcript torture page must mutate a large variable-height transcript through the real
  UI Gallery runtime while keeping diagnostics assertions stable. Because `fret-ui-ai` transcript
  surfaces intentionally use non-retained virtual lists, the suite must not apply retained-window
  reconcile tail checks to this script.
- finding:
  no AI transcript scroll mutation defect was reproduced. The first suite rerun after strengthening
  the script showed all three scripts passing while the suite still failed
  `tooling.suite.success_tail.failed`; the tail artifact showed a retained-only
  non-retained-shift check was being applied to the transcript torture script. Removing the script
  from `ui_gallery_script_requires_retained_vlist_reconcile_gate` matches the `fret-ui-ai`
  non-static surface policy and eliminates the false suite failure.
- diagnostics surface:
  `ui-gallery-ai-transcript-torture-scroll.json` injects
  `FRET_UI_GALLERY_AI_TRANSCRIPT_LEN=240` and
  `FRET_UI_GALLERY_AI_TRANSCRIPT_VARIABLE_HEIGHT=1`, asserts the hidden
  `ui-gallery-ai-transcript-messages-len` semantics value is `240`, clicks the append control,
  asserts the value becomes `340`, and captures layout, screenshot, and bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/ai/transcript_torture.rs`,
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-ai-transcript-torture-scroll.json`,
  `crates/fret-diag/src/diag_policy.rs`,
  `crates/fret-diag/src/tests.rs`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`, and
  `ecosystem/fret-ui-ai/src/surface_policy_tests.rs`.
- evidence anchors:
  focused AI packet
  `target/fret-diag-ai-transcript-torture-count-gate-v1/sessions/1779201370619-115172/1779201476652/ai.packet`;
  focused pack
  `target/fret-diag-ai-transcript-torture-count-gate-v1/sessions/1779201370619-115172/share/1779201476652.zip`;
  suite summary
  `target/fret-diag-ai-transcript-retained-cargo-policy-v2/sessions/1779203319147-101240/suite.summary.json`.
- format/registry:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\ai\transcript_torture.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs crates\fret-diag\src\diag_policy.rs crates\fret-diag\src\tests.rs`
  - result: passed.
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- diag policy regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag ai_transcript_torture_scroll_is_not_a_retained_vlist_reconcile_gate --no-fail-fast --no-capture`
  - result: passed; Nextest run id `caf72dfa-d836-47df-8dd3-0aa22a1618e5`.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_ai_conversation_demo_screenshot_zinc_dark script_v2_roundtrip_ui_gallery_ai_conversation_demo_scroll_button script_v2_roundtrip_ui_gallery_ai_transcript_torture_scroll --no-fail-fast --no-capture`
  - result: passed; Nextest run id `674f2827-c68d-4532-917f-583e0e81cc1b`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\perf\ui-gallery-ai-transcript-torture-scroll.json --dir target\fret-diag-ai-transcript-torture-count-gate-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779201476652`.
- full AI transcript suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-ai-transcript-retained --dir target\fret-diag-ai-transcript-retained-cargo-policy-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed after rebuilding `fretboard-dev`; 3/3 scripts; torture run id
    `1779203465969`; no `check.vlist_window_shifts_non_retained_max.json` tail file was produced.

## AI Transcript Append Window Refresh Gate

- invariant:
  the AI transcript torture page must materialize appended transcript rows in the final diagnostics
  bundle after the append mutation and a stable scroll refresh. The surface remains non-retained,
  so this gate must not rely on retained-window reconcile tail policy.
- finding:
  no AI transcript mechanism defect was reproduced. The new companion gate proves row-8 and row-9
  both appear in the final bundle after appending messages to the small transcript, then scrolling
  the transcript root and capturing the layout/bundle pair.
- diagnostics surface:
  `ui-gallery-ai-transcript-append-window-refresh.json` starts on `ai_transcript_torture`, uses an
  8-message variable-height transcript, appends 100 messages, scrolls the root, waits for
  `ui-gallery-ai-transcript-row-8` and `ui-gallery-ai-transcript-row-9`, and captures a layout
  sidecar plus bundle evidence.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-transcript-append-window-refresh.json`,
  `tools/diag-scripts/suites/ui-gallery-ai-transcript-retained/suite.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`, and
  `tools/diag-scripts/index.json`.
- evidence anchors:
  focused AI packet
  `target/fret-diag-ai-transcript-append-window-refresh-v2/sessions/1779479206565-76328/1779479217864/ai.packet`;
  focused pack
  `target/fret-diag-ai-transcript-append-window-refresh-v2/sessions/1779479206565-76328/share/1779479217864.zip`;
  suite summary
  `target/fret-diag-ai-transcript-retained-suite-append-window-refresh-v3/sessions/1779479378187-25176/suite.summary.json`.
- runtime proof:
  `diag query test-id` on the focused bundle returns `ui-gallery-ai-transcript-row-8` and
  `ui-gallery-ai-transcript-row-9` once each; `diag slice` for `ui-gallery-ai-transcript-row-8`
  shows the row in frame 28 under `ui-gallery-ai-transcript-root`.
- format/registry/protocol:
  `python -m json.tool tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-transcript-append-window-refresh.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-ai-transcript-retained/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_ai_transcript_append_window_refresh --no-fail-fast --no-capture`
  - result: passed; Nextest run id `4140c0a8-8c78-4ad5-a5b2-39c42ed57bd1`.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-transcript-append-window-refresh.json --dir target/fret-diag-ai-transcript-append-window-refresh-v2 --session-auto --pack --ai-packet --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779479217864`.
- runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-ai-transcript-retained --dir target/fret-diag-ai-transcript-retained-suite-append-window-refresh-v3 --session-auto --include-triage --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed 4/4; the new append-window-refresh row run id was `1779479605022`.

## Combobox RTL Long Text Startup Intro Non-Overlap Gate

- invariant:
  the Combobox docs intro must reserve enough measured vertical space before the focused
  `RTL Long Text` section title on cold startup, before a manual resize can repair stale text
  layout.
- finding:
  the latest user screenshot corrected the target from `Popup` to `RTL Long Text`: the title
  overlapped the long docs intro while the page was focused on that section. Current `dev-fast`
  diagnostics did not reproduce the overlap after the prior text repair-frame clipping work, but
  the old gate set had no direct RTL Long Text intro/title startup assertion.
- diagnostics surface:
  `ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json` starts with
  `FRET_UI_GALLERY_START_SECTION=RTL Long Text` and `FRET_UI_GALLERY_MAIN_WINDOW_SIZE=1083x752`,
  captures layout/screenshot/bundle evidence, asserts intro/title non-overlap, asserts a `>= 16px`
  intro-to-title gap, and asserts a `>= 8px` title-to-description gap. The companion
  `ui-gallery-combobox-popup-doc-intro-logical994-startup-non-overlap.json` keeps the earlier
  physical-size interpretation covered for Popup.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json`,
  `tools/diag-scripts/ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json`,
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-doc-intro-logical994-startup-non-overlap.json`,
  `tools/diag-scripts/ui-gallery-combobox-popup-doc-intro-logical994-startup-non-overlap.json`,
  `tools/diag-scripts/suites/ui-gallery-combobox-geometry-placement/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused RTL Long Text AI packet
  `target/fret-diag-combobox-rtl-long-text-doc-intro-logical1083-gate-v1/sessions/1779207086203-106128/1779207094769/ai.packet`;
  focused RTL Long Text pack
  `target/fret-diag-combobox-rtl-long-text-doc-intro-logical1083-gate-v1/sessions/1779207086203-106128/share/1779207094769.zip`;
  full suite summary
  `target/fret-diag-combobox-geometry-placement-rtl-long-text-v1/sessions/1779208245269-120048/suite.summary.json`.
- JSON/registry/format:
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json > $null`,
  `python -m json.tool tools\diag-scripts\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json > $null`,
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-combobox-geometry-placement\suite.json > $null`,
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`,
  and `python tools\check_diag_scripts_registry.py`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_popup_doc_intro_logical994_startup_non_overlap script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_doc_intro_logical1083_startup_non_overlap --no-fail-fast --no-capture`
  - result: passed; Nextest run id `6619d838-cd48-41d2-b279-ede4466fc291`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-doc-intro-logical1083-gate-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779207094769`.
- full Combobox geometry placement suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-rtl-long-text-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; 12/12 scripts; RTL Long Text startup run id `1779208395010`; Popup logical994
    run id `1779208377600`.

## First-Paint Text Auto-Height Repair

- invariant:
  text nodes with auto height must not visibly overlap following content when their first
  paint-time prepared metrics are taller than stale startup layout bounds. The repair must schedule
  another layout frame and clip the taller text on the stale frame.
- finding:
  the M140 RTL Long Text startup script pressed `Escape` before capturing evidence. That keyboard
  input advanced a frame and could hide the exact cold-start path reported by the user. Removing
  the input produced a stricter startup gate; before the mechanism fix, one focused run stalled at
  screenshot capture with `script_stalled_no_frames`, showing the startup path could fail to drive
  itself without input.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/paint.rs`,
  `crates/fret-ui/src/declarative/tests/text_cache.rs`, and
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json`.
- evidence anchors:
  pre-fix no-input failed run:
  `target/fret-diag-combobox-rtl-long-text-no-input-repro-v1/sessions/1779209752041-10556/script.result.json`;
  fixed focused AI packet:
  `target/fret-diag-combobox-rtl-long-text-no-input-fixed-v1/sessions/1779210358866-112204/1779210456769/ai.packet`;
  fixed focused screenshot:
  `target/fret-diag-combobox-rtl-long-text-no-input-fixed-v1/sessions/1779210358866-112204/screenshots/1779210460025-ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap/window-4294967297-tick-2-frame-2.png`;
  full suite summary:
  `target/fret-diag-combobox-geometry-placement-startup-text-repair-v1/sessions/1779210565472-66488/suite.summary.json`.
- format:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\paint.rs crates\fret-ui\src\declarative\tests\text_cache.rs`
  - result: passed.
- mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_first_paint_reinvalidates_layout_when_height_grows wrapped_text_paint_width_shrink_reinvalidates_layout_when_height_grows --no-fail-fast --no-capture`
  - result: passed; Nextest run id `ee45c3ee-bd9e-4983-bf51-3a676fe8efdc`.
- protocol/registry:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_doc_intro_logical1083_startup_non_overlap --no-fail-fast --no-capture`
  - result: passed; Nextest run id `af2bdc87-44c4-4be2-8e92-d5a6a062da39`.
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-no-input-fixed-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779210456769`; the captured screenshot shows clean intro/title
    spacing without keyboard or resize recovery.
- full Combobox geometry placement suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-startup-text-repair-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; 12/12 scripts; no-input RTL Long Text startup run id `1779210808250`.

## Wrapped Text Prepared Measurement Convergence

- invariant:
  wrapped text startup measurement must reserve the same height as the prepared text blob used for
  paint. Layout-bounds assertions are insufficient when actual painted ink can exceed stale
  measured bounds.
- finding:
  the remaining Combobox RTL Long Text screenshot matched a measure/prepare divergence risk:
  `TextService::measure` could underestimate wrapped height, while paint used a taller prepared
  blob. Resize then corrected layout by forcing a fresh measurement/preparation path.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/measure.rs`,
  `crates/fret-ui/src/declarative/tests/text_cache.rs`, and
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json`.
- evidence anchors:
  fixed focused AI packet:
  `target/fret-diag-combobox-rtl-long-text-startup-prepared-measure-v3/sessions/1779215091187-112692/1779215099640/ai.packet`;
  fixed focused pack:
  `target/fret-diag-combobox-rtl-long-text-startup-prepared-measure-v3/sessions/1779215091187-112692/share/1779215099640.zip`;
  fixed focused screenshot:
  `target/fret-diag-combobox-rtl-long-text-startup-prepared-measure-v3/sessions/1779215091187-112692/screenshots/1779215103570-ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap/window-4294967297-tick-3-frame-3.png`.
- format:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\measure.rs crates\fret-ui\src\declarative\tests\text_cache.rs`
  - result: passed.
- mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_measure_uses_prepare_metrics_for_startup_layout wrapped_text_first_paint_reinvalidates_layout_when_height_grows theme_color_change_does_not_change_text_input_fingerprints --no-fail-fast --no-capture`
  - result: passed; Nextest run id `c70a4417-6ee8-46f1-bc4f-a485bc98a122`.
- registry:
  `python tools\check_diag_scripts_registry.py`
  - result: passed; registry is up to date.
- build:
  `cargo build --profile dev-fast -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-startup-prepared-measure-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779215099640`; the gate now asserts intro/title,
    title/description, description/content, and description/trigger spacing.

## Chart Torture Multi-Series Tooltip Output

- invariant:
  retained chart output must keep tooltip rows fresh for every visible series after real pan/zoom
  interaction, while the domain-window output must follow ADR 0301 link-key uniqueness rules.
- finding:
  no retained chart tooltip defect was reproduced. The first multi-series runtime run exposed a
  stale diagnostics oracle instead: `domain_windows_count == 2` was invalid once the page had two Y
  fields on one Y axis. ADR 0301 only auto-exports axes that resolve to one `(dataset, field)`, so
  the ambiguous Y axis should not be exported without an explicit host map.
- diagnostics surface:
  `ui-gallery-chart-torture-pan-zoom.json` now drives the existing retained Chart Torture pan/zoom
  path with two line series. It asserts the output model publishes one X domain window, that the X
  output window matches dataZoom and changes from the full domain, and that the tooltip has one
  axis header, two source-owned series rows, labels `A` and `B`, and zero missing rows.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs`,
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`, and
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-chart-torture-pan-zoom.json`.
- evidence anchors:
  stale-oracle failure bundle
  `target/fret-diag-chart-torture-multiseries-tooltip-v2/sessions/1779216523336-80372/1779216691687-script-step-0026-wait_until-timeout/bundle.schema2.json`;
  fixed focused AI packet
  `target/fret-diag-chart-torture-multiseries-tooltip-v3/sessions/1779217007250-123724/1779217026347/ai.packet`;
  fixed focused pack
  `target/fret-diag-chart-torture-multiseries-tooltip-v3/sessions/1779217007250-123724/share/1779217026347.zip`;
  suite summary
  `target/fret-diag-chart-torture-suite-multiseries-tooltip-v1/sessions/1779217110888-98424/suite.summary.json`.
- format/JSON/registry:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\driver\diag_snapshot.rs apps\fret-ui-gallery\src\ui\previews\pages\torture\chart_torture.rs`
  - result: passed.
  `python -m json.tool tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json > $null`
  - result: passed.
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_pan_zoom --no-fail-fast --no-capture`
  - result: passed; Nextest run id `993eeccd-72d1-49f4-830f-a710b0b16250`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-pan-zoom.json --dir target\fret-diag-chart-torture-multiseries-tooltip-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779217026347`.
- full Chart Torture suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target\fret-diag-chart-torture-suite-multiseries-tooltip-v1 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; script run id `1779217122878`.

## Cached Prepared Text Stale-Bounds Repair

- invariant:
  wrapped auto-height text must not paint cached prepared glyphs outside stale startup bounds while
  waiting for a follow-up layout frame.
- finding:
  a later manual Combobox RTL Long Text screenshot still showed visible overlap. Existing runtime
  gates were frame-3 layout/screenshot gates, so they could pass after repair and still miss the
  first-visible-frame cached prepared paint path. The missing mechanism was that
  `maybe_repair_text_layout_after_paint_prepare` only ran inside `needs_prepare`; layout-prepared
  cached blobs could skip the stale-bounds repair check.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/paint.rs` and
  `crates/fret-ui/src/declarative/tests/text_cache.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-ui-gallery-combobox-rtl-intro-overlap-fixed-1624-v1/sessions/1779219328227-121516/1779219333574/ai.packet`;
  focused runtime pack:
  `target/fret-diag-ui-gallery-combobox-rtl-intro-overlap-fixed-1624-v1/sessions/1779219328227-121516/share/1779219333574.zip`.
- format:
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\paint.rs crates\fret-ui\src\declarative\tests\text_cache.rs`
  - result: passed.
- mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui wrapped_text_cached_prepared_metrics_reinvalidate_when_bounds_height_shrinks --no-fail-fast --no-capture`
  - pre-fix result: failed; the cached prepared metrics path did not schedule layout repair.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui text_cache --no-fail-fast --no-capture`
  - fixed result: passed; Nextest run id `c4dc5647-ab06-4015-be4a-829f175a3359`.
- build:
  `cargo build --profile dev-fast -p fret-ui-gallery`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-ui-gallery-combobox-rtl-intro-overlap-fixed-1624-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779219333574`.

## Combobox RTL Long Text Doc Scaffold Min-Width Clamp

- invariant:
  UI Gallery doc scaffold text that fills a card/flex content column must also opt into
  `min-width: 0`, so startup wrapped-text measurement uses the resolved content width instead of
  an over-wide first pass that can make following section content overlap until resize recovery.
- finding:
  the latest manual screenshot still showed `RTL Long Text` visually colliding with the docs intro.
  The Combobox trigger was not the source; the remaining issue was in shared doc scaffold helpers:
  `muted_full_width` and `section_title` set fill width but did not set `min_width=0`.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/doc_layout.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap-v2/sessions/1779230951009-124292/1779230956616/ai.packet`;
  clean startup screenshot:
  `target/fret-diag-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap-v2/sessions/1779230951009-124292/screenshots/1779230959201-ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap/window-4294967297-tick-3-frame-3.png`.
- unit regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery doc_text_helpers_keep_fill_width_min_w_zero --no-fail-fast --no-capture`
  - result: passed; Nextest run id `3c572126-8add-42ad-8fc7-9b02766e5ba3`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-doc-intro-logical1083-startup-non-overlap-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779230956616`.

## Chart Explicit Link-Axis Mapping Output Gate

- invariant:
  ADR 0301's conservative auto mapping should omit an ambiguous shared Y axis, but an explicit
  host-provided `AxisId -> LinkAxisKey` map must publish that Y domain window to
  `ChartCanvasOutput`.
- finding:
  no retained chart output defect was reproduced. The missing coverage was the explicit-map
  companion to F226's ambiguous-Y conservative path. The first attempt to place the gate in the
  existing Chart Torture suite also exposed a diagnostics suite composition hazard: the pan/zoom
  suite's `chart_sampling_window_shifts_min` tail check is not valid for an explicit-output-only
  script.
- implementation anchors:
  `ecosystem/fret-chart/src/retained/canvas.rs`,
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs`,
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-chart-torture-explicit-y-link-map.json`,
  `tools/diag-scripts/suites/ui-gallery-chart-linking-explicit-y-map/suite.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`, and
  `crates/fret-diag/src/diag_suite.rs`.
- evidence anchors:
  focused explicit-Y AI packet:
  `target/fret-diag-chart-torture-explicit-y-link-map-v2/sessions/1779221804831-35760/1779221899196/ai.packet`;
  focused explicit-Y pack:
  `target/fret-diag-chart-torture-explicit-y-link-map-v2/sessions/1779221804831-35760/share/1779221899196.zip`;
  suite summary:
  `target/fret-diag-chart-linking-explicit-y-map-suite-v1/sessions/1779226956912-131628/suite.summary.json`;
  original suite recheck summary:
  `target/fret-diag-chart-torture-suite-recheck-v1/sessions/1779226999698-96944/suite.summary.json`;
  suite policy hazard summary:
  `target/fret-diag-chart-torture-suite-explicit-y-link-map-v1/sessions/1779222142969-22864/suite.summary.json`.
- format/JSON/registry:
  `rustfmt --edition 2024 --check crates\fret-diag\src\diag_suite.rs apps\fret-ui-gallery\src\driver\diag_snapshot.rs apps\fret-ui-gallery\src\ui\previews\pages\torture\chart_torture.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs ecosystem\fret-chart\src\retained\canvas.rs`
  - result: passed.
  `python -m json.tool tools\diag-scripts\ui-gallery\perf\ui-gallery-chart-torture-explicit-y-link-map.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-chart-torture-explicit-y-link-map.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-chart-linking-explicit-y-map\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-chart-torture\suite.json > $null`
  - result: passed.
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- focused Rust checks:
  `cargo nextest run --cargo-profile dev-fast -p fret-chart explicit_link_axis_map_publishes_ambiguous_y_domain_window_to_output_model --no-fail-fast --no-capture`
  - result: passed; Nextest run id `6d65c626-9933-45ca-b30b-e15ce835bd83`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_torture_explicit_y_link_map --no-fail-fast --no-capture`
  - result: passed; Nextest run id `1c0b302d-5894-4a6c-a90a-8e4505d72c2e`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag build_suite_core_default_post_run_checks_keeps_chart_linking_explicit_y_map_generic --no-fail-fast --no-capture`
  - result: passed; Nextest run id `9225e20a-b2db-445c-aae1-ef9e369cac20`.
- runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-chart-linking-explicit-y-map --dir target\fret-diag-chart-linking-explicit-y-map-suite-v1 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779226972500`.
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-chart-torture --dir target\fret-diag-chart-torture-suite-recheck-v1 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-chart,gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779227011824`.

## Combobox RTL Long Text Client-Height Startup Gate

- invariant:
  the focused Combobox `RTL Long Text` docs section must not overlap the long docs intro at both the
  canonical screenshot logical client size and the shorter client-area interpretation of a decorated
  Windows screenshot.
- finding:
  a follow-up manual screenshot still showed visible overlap, but fresh `target\dev-fast`
  diagnostics did not reproduce it. The most likely ambiguity was screenshot geometry: the manual
  image included the native title bar, while diagnostics screenshots use the drawable client area.
  The new gate locks a `1083x721` client size, producing a `1625x1082` physical screenshot at 1.5x.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json`,
  `tools/diag-scripts/ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json`,
  `tools/diag-scripts/suites/ui-gallery-combobox-geometry-placement/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused client-height AI packet:
  `target/fret-diag-combobox-rtl-long-text-client721-gate-v1/sessions/1779232796961-55416/1779232803236/ai.packet`;
  focused client-height pack:
  `target/fret-diag-combobox-rtl-long-text-client721-gate-v1/sessions/1779232796961-55416/share/1779232803236.zip`;
  full suite summary:
  `target/fret-diag-combobox-geometry-placement-client721-v1/sessions/1779232841519-125836/suite.summary.json`.
- JSON/registry/format:
  `python -m json.tool tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-combobox-geometry-placement\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_combobox_rtl_long_text_doc_intro_client721_startup_non_overlap --no-fail-fast --no-capture`
  - result: passed; Nextest run id `6814997f-e496-4dff-82a5-2c30636c7c54`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-client721-gate-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779232803236`; screenshot size `1625x1082`.
- full runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-combobox-geometry-placement --dir target\fret-diag-combobox-geometry-placement-client721-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; 13/13 rows; new client-height run id `1779232938320`.
- residual note:
  `cargo build -p fret-ui-gallery` was attempted to refresh `target\debug\fret-ui-gallery.exe`, but
  timed out after five minutes and was stopped. Do not treat `target\debug` or older
  `target\release` gallery binaries as evidence for this slice unless rebuilt separately.

## Chart Explicit Y Linked-Domain Propagation Mechanism Gate

- invariant:
  an explicit Y link-axis map must propagate through the linked-domain shared model into a second
  retained chart, not only publish from the source chart's output model.
- finding:
  no `fret-chart` mechanism defect was reproduced. The missing coverage was the second half of the
  propagation chain after F228: source output publication had a gate, but the target chart's real
  paint-time `sync_linked_domain_windows` path and target output publication were not covered.
- implementation anchors:
  `ecosystem/fret-chart/src/retained/canvas.rs`.
- test shape:
  `explicit_y_domain_window_propagates_to_second_linked_chart_output_model` creates source and
  target retained charts from the ambiguous multi-axis spec, applies an explicit Y map for
  `AxisId::new(3) -> LinkAxisKey { kind: Y, dataset: 1, field: 2 }`, publishes source output,
  ticks `LinkedChartGroup`, pumps real target retained layout/paint frames, and asserts target
  output publishes the propagated `[-0.25, 0.75]` window instead of its initial `[-5.0, 5.0]`
  local Y window.
- format:
  `rustfmt --edition 2024 --check ecosystem\fret-chart\src\retained\canvas.rs`
  - result: passed.
- mechanism regressions:
  `cargo nextest run --cargo-profile dev-fast -p fret-chart explicit_link_axis_map_publishes_ambiguous_y_domain_window_to_output_model explicit_y_domain_window_propagates_to_second_linked_chart_output_model --no-fail-fast --no-capture`
  - result: passed; Nextest run id `620beddb-8a62-4de0-81fd-d5f2fadb28f1`.
- residual runtime gap:
  `apps/fret-examples/src/chart_multi_axis_demo.rs` already has two linked charts and deterministic
  diagnostics auto-zoom, but currently exposes linked-domain state through logs rather than an app
  snapshot provider. Add a bounded snapshot surface before promoting a runtime assertion gate.

## Fixed-Line-Box Cold Word-Wrap Startup Repair

- invariant:
  a fixed-line-height wrapped paragraph must not let the internal `Hg` line-metrics probe overwrite
  the paragraph layout being returned for paint. Cold-process startup must reserve and draw the real
  wrapped paragraph on the first visible frame.
- finding:
  the manual Combobox RTL Long Text overlap remained valid after the previous client-height gate.
  A temporary frame-1 probe showed the deeper mechanism defect: the docs intro rendered as `Hg`,
  which is the internal fixed line-box metrics sample, before later frames converged to the real
  paragraph. `ParleyShaper::shape_paragraph_with_wrap` computed fixed-line-box metrics after
  building the paragraph into the shared Parley layout, so a cold `shape_single_line_metrics("Hg")`
  call clobbered that layout before the paragraph lines were consumed.
- implementation anchors:
  `crates/fret-render-text/src/parley_shaper.rs`.
- evidence anchors:
  fixed `dev-fast` first-frame screenshot:
  `target/fret-diag-combobox-rtl-long-text-devfast-frame1-fixed-v1/sessions/1779237666634-128156/screenshots/1779237681089-ui-gallery-combobox-rtl-long-text-client721-frame1-screenshot/window-4294967297-tick-4-frame-4.png`;
  fixed `dev-fast` focused client-height AI packet:
  `target/fret-diag-combobox-rtl-long-text-devfast-client721-fixed-v1/sessions/1779237666672-136820/1779237680925/ai.packet`;
  fixed debug focused client-height AI packet:
  `target/fret-diag-combobox-rtl-long-text-debug-client721-fixed-v1/sessions/1779238353963-52416/1779238359542/ai.packet`.
- format:
  `rustfmt --edition 2024 --check crates\fret-render-text\src\parley_shaper.rs`
  - result: passed.
- mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-render-text --no-fail-fast --no-capture`
  - result: passed; 85/85 tests; Nextest run id `a16c3aa8-c5dc-4b48-a26f-df17e39f442e`.
- gallery rebuilds:
  `cargo build --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed.
  `cargo build -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-devfast-client721-fixed-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779237680925`.
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-rtl-long-text-doc-intro-client721-startup-non-overlap.json --dir target\fret-diag-combobox-rtl-long-text-debug-client721-fixed-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\debug\fret-ui-gallery.exe`
  - result: passed; run id `1779238359542`.
- registry/checks:
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
  `git diff --check`
  - result: passed.
- binary freshness note:
  `target\debug\fret-ui-gallery.exe` and `target\dev-fast\fret-ui-gallery.exe` were rebuilt after
  the fix. `target\release\fret-ui-gallery.exe` is still from 2026-05-14 and was not used as
  evidence.

## Chart Multi-Axis Linked-Domain Runtime Snapshot Gate

- invariant:
  the live `chart_multi_axis_demo` shell must propagate a top-chart X domain-window change through
  `LinkedChartGroup` into the shared linked-domain model and the bottom chart output model. The
  runtime gate must assert state, not only pixels or logs.
- finding:
  no runtime linked-domain defect was reproduced. The missing piece was an app snapshot surface:
  the demo already applies a deterministic diagnostics-only top-chart X window change to
  `[-75, 75]`, but prior evidence could only observe logs/pixels rather than shared/top/bottom
  `ChartCanvasOutput` state.
- implementation anchors:
  `apps/fret-examples/src/chart_multi_axis_demo.rs`,
  `tools/diag-scripts/charts/chart-multi-axis-linked-domain-window-app-snapshot.json`,
  `tools/diag-scripts/suites/chart-multi-axis-linking/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-chart-multi-axis-linked-domain-app-snapshot-v1/sessions/1779239502304-133288/1779239505892/ai.packet`;
  focused runtime pack:
  `target/fret-diag-chart-multi-axis-linked-domain-app-snapshot-v1/sessions/1779239502304-133288/share/1779239505892.zip`;
  suite summary:
  `target/fret-diag-chart-multi-axis-linking-suite-v1/sessions/1779239623009-133816/suite.summary.json`.
- format/JSON/registry:
  `rustfmt --edition 2024 --check apps\fret-examples\src\chart_multi_axis_demo.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `python -m json.tool tools\diag-scripts\charts\chart-multi-axis-linked-domain-window-app-snapshot.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\chart-multi-axis-linking\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- build:
  `cargo build --profile dev-fast -p fret-demo --bin chart_multi_axis_demo`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_chart_multi_axis_linked_domain_window_app_snapshot --no-fail-fast --no-capture`
  - result: passed; Nextest run id `1872c4bc-48ce-4a41-a564-ed9f74f83461`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\charts\chart-multi-axis-linked-domain-window-app-snapshot.json --dir target\fret-diag-chart-multi-axis-linked-domain-app-snapshot-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\chart_multi_axis_demo.exe`
  - result: passed; run id `1779239505892`.
- suite runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite chart-multi-axis-linking --dir target\fret-diag-chart-multi-axis-linking-suite-v1 --session-auto --timeout-ms 420000 --launch -- target\dev-fast\chart_multi_axis_demo.exe`
  - result: passed; run id `1779239625616`.

## Item Cold Startup Long-Docs Text Runtime Gate

- invariant:
  fixed-line-height wrapped docs text must reserve the real cold-start paragraph height outside the
  original Combobox repro page. A non-Combobox docs page should not regress to one-line startup
  measurement or let a following section title overlap the intro before resize recovery.
- finding:
  the latest manual Combobox screenshot remained a valid user-observed symptom, but current rebuilt
  Gallery binaries did not reproduce it. `target\release\fret-ui-gallery.exe` was discovered to be
  an older 2026-05-14 artifact before rebuild, so runtime evidence for this slice uses a refreshed
  release binary. The coverage gap was therefore adjacent runtime breadth, not a new confirmed text
  mechanism defect.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/item/ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap.json`,
  `tools/diag-scripts/ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-item-vs-field-client721-startup-non-overlap-v1/sessions/1779242677744-141216/1779242679435/ai.packet`;
  focused runtime screenshot:
  `target/fret-diag-item-vs-field-client721-startup-non-overlap-v1/sessions/1779242677744-141216/screenshots/1779242679542-ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap/window-4294967297-tick-3-frame-3.png`;
  suite summary:
  `target/fret-diag-ui-gallery-shadcn-runtime-evidence-item-vs-field-v1/sessions/1779242784682-41468/suite.summary.json`.
- JSON/registry/format:
  `python -m json.tool tools\diag-scripts\ui-gallery\item\ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_item_vs_field_doc_intro_client721_startup_non_overlap --no-fail-fast --no-capture`
  - result: passed; Nextest run id `ee8a6ea4-70e4-4a81-af24-980b7b1f603c`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\item\ui-gallery-item-vs-field-doc-intro-client721-startup-non-overlap.json --dir target\fret-diag-item-vs-field-client721-startup-non-overlap-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\release\fret-ui-gallery.exe`
  - result: passed; run id `1779242679435`.
- suite runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-ui-gallery-shadcn-runtime-evidence-item-vs-field-v1 --session-auto --timeout-ms 900000 --launch -- target\release\fret-ui-gallery.exe`
  - result: passed; new script run id `1779242852622`.

## View Cache Dynamic Text Mutation Runtime Gate

- invariant:
  a cached View Cache subtree must refresh visible wrapped text, prepared-text cache state, and
  following layout when its observed counter model changes. The runtime gate must assert visible
  text and geometry, not only the app-snapshot counter.
- finding:
  no new view-cache, text-cache, or layout defect was reproduced. The coverage gap was that the
  previous View Cache gate covered counter and Popover state without changing a wrapped text leaf
  inside the cached subtree. The first focused runtime draft exposed an over-constrained script
  oracle: Popover trigger and retained list are adjacent under current CardContent semantics, so the
  durable assertion for that pair is non-overlap rather than an `8px` gap.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/harness/view_cache.rs`,
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
  `tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-dynamic-text-mutation-through-cache.json`,
  `tools/diag-scripts/ui-gallery-view-cache-dynamic-text-mutation-through-cache.json`,
  `tools/diag-scripts/suites/ui-gallery-view-cache/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  first over-constrained failure:
  `target/fret-diag-ui-gallery-view-cache-dynamic-text-mutation-v1/sessions/1779244450932-17084/script.result.json`;
  focused runtime AI packet:
  `target/fret-diag-ui-gallery-view-cache-dynamic-text-mutation-v2/sessions/1779244725576-99248/1779244734657/ai.packet`;
  focused runtime pack:
  `target/fret-diag-ui-gallery-view-cache-dynamic-text-mutation-v2/sessions/1779244725576-99248/share/1779244734657.zip`;
  suite summary:
  `target/fret-diag-ui-gallery-view-cache-suite-dynamic-text-v1/sessions/1779244758600-135808/suite.summary.json`.
- JSON/registry/format:
  `python -m json.tool tools\diag-scripts\ui-gallery\view-cache\ui-gallery-view-cache-dynamic-text-mutation-through-cache.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-view-cache-dynamic-text-mutation-through-cache.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-view-cache\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\previews\pages\harness\view_cache.rs apps\fret-ui-gallery\src\driver\diag_snapshot.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_view_cache_dynamic_text_mutation_through_cache --no-fail-fast --no-capture`
  - result: passed; Nextest run id `bd7f6552-74b2-416f-b0f0-55bb8f82742f`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\view-cache\ui-gallery-view-cache-dynamic-text-mutation-through-cache.json --dir target\fret-diag-ui-gallery-view-cache-dynamic-text-mutation-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779244734657`.
- suite runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-view-cache --dir target\fret-diag-ui-gallery-view-cache-suite-dynamic-text-v1 --session-auto --timeout-ms 600000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; 2/2 rows; new script run id `1779244796106`.


## Provider-Sensitive ViewCache and Direction Runtime Gate

- invariant:
  provider-style inherited state such as shadcn/Radix `DirectionProvider` is recipe policy, not an
  implicit mechanism-layer external dependency. A cached subtree that reads provider state must use
  an explicit `ViewCacheProps::cache_key` when provider changes should rebuild that subtree; an
  unkeyed cached subtree keeps replaying the first rendered provider-sensitive output.
- finding:
  no new mechanism or recipe defect was reproduced. The gap was contract evidence: the ViewCache
  lifecycle fixture covered cache keys and external environment/layout deps, while Direction had a
  docs smoke script but no promoted runtime suite. The new fixture cases document both the safe
  explicit-key path and the intentional unkeyed reuse hazard for DirectionProvider-like state.
- implementation anchors:
  `crates/fret-ui/src/declarative/tests/fixtures/view_cache_lifecycle_v1.json`,
  `crates/fret-ui/src/declarative/tests/view_cache_lifecycle_harness.rs`,
  `crates/fret-ui/src/declarative/tests/view_cache.rs`,
  `tools/diag-scripts/ui-gallery/direction/ui-gallery-direction-docs-smoke.json`,
  `tools/diag-scripts/suites/ui-gallery-direction/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  Direction suite summary:
  `target/fret-diag-ui-gallery-direction-suite-provider-v1/sessions/1779420909017-199712/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/view_cache_lifecycle_v1.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/direction/ui-gallery-direction-docs-smoke.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-direction/suite.json > $null`;
  `python -m json.tool tools/diag-scripts/index.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates/fret-core/src/layout_direction.rs crates/fret-ui/src/declarative/tests/view_cache.rs crates/fret-ui/src/declarative/tests/view_cache_lifecycle_harness.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_view_cache_lifecycle_matches_oracles view_cache_explicit_cache_key_tracks_provider_state_changes view_cache_provider_state_changes_without_cache_key_keep_reusing_documented_contract --no-fail-fast`
  - result: passed; run id `af739255-9dbe-4cd7-b397-673b7dc1415e`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_direction_docs_smoke --no-fail-fast`
  - result: passed; run id `fac17852-1fe3-43f3-99e3-f371a17b889e`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- dedicated runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-direction --dir target/fret-diag-ui-gallery-direction-suite-provider-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 1/1; Direction docs-smoke run id `1779421149267`.

## HitTestOnly Paint-Cache Replay Runtime Gate

- invariant:
  pointer movement over a stable cached visual surface may invalidate hit testing without forcing a
  paint-cache key change. A runtime gate must prove the `HitTestOnly` path actually reaches replay
  and that stable geometry does not accumulate key-mismatch rejections.
- finding:
  the existing script was not a durable gate. It waited for the probe region on whichever page UI
  Gallery opened by default, so the first focused run timed out on the Overlay page. After adding
  navigation, the first app-snapshot assertion used the wrong `/shell/selected_page` pointer; the
  corrected UI Gallery snapshot pointer is `/selected_page`. Promoting the script into a
  zero-warning suite also discovered duplicate `ui-gallery-hit-test-only-probe-region` ids in the
  page. The owning page now separates the outer panel id from the inner hit region id.
- implementation anchors:
  `crates/fret-diag-protocol/src/lib.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/frame_stats.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`,
  `apps/fret-ui-gallery/src/ui/previews/pages/harness/hit_test_only_paint_cache_probe.rs`,
  `tools/diag-scripts/ui-gallery/diag/ui-gallery-hit-test-only-paint-cache-probe-sweep.json`,
  `tools/diag-scripts/suites/ui-gallery-hit-test-only-paint-cache/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  baseline timeout before navigation:
  `target/fret-diag-hit-test-only-paint-cache-probe-baseline-inspect/sessions/1779245360940-54392/script.result.json`;
  wrong app-snapshot pointer failure:
  `target/fret-diag-hit-test-only-paint-cache-probe-sweep-v1/sessions/1779247395509-143424/script.result.json`;
  duplicate-id lint failure:
  `target/fret-diag-hit-test-only-paint-cache-suite-v1/sessions/1779248014312-104692/1779248135656-ui-gallery-hit-test-only-paint-cache-probe-sweep/check.lint.json`;
  passing focused runtime AI packet:
  `target/fret-diag-hit-test-only-paint-cache-probe-sweep-v2/sessions/1779247851157-129852/1779247865248/ai.packet`;
  passing focused runtime pack:
  `target/fret-diag-hit-test-only-paint-cache-probe-sweep-v2/sessions/1779247851157-129852/share/1779247865248.zip`;
  passing suite summary:
  `target/fret-diag-hit-test-only-paint-cache-suite-v3/sessions/1779249174760-142600/suite.summary.json`.
- JSON/registry:
  `python -m json.tool tools\diag-scripts\ui-gallery\diag\ui-gallery-hit-test-only-paint-cache-probe-sweep.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-hit-test-only-paint-cache\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- format/static checks:
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\previews\pages\harness\hit_test_only_paint_cache_probe.rs crates\fret-diag-protocol\src\lib.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs ecosystem\fret-bootstrap\src\ui_diagnostics\debug_snapshot_predicates.rs ecosystem\fret-bootstrap\src\ui_diagnostics\predicates.rs`;
  `git diff --check`
  - result: passed. `frame_stats.rs` and `service.rs` are intentionally excluded from file-wide
    rustfmt because their existing formatting drifts outside this slice; the diff only forwards the
    new counters and predicate arms.
- protocol roundtrip and predicate serialization:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_paint_cache_hit_test_only_replay_counters_serialize script_v2_roundtrip_ui_gallery_hit_test_only_paint_cache_probe_sweep --no-fail-fast --no-capture`
  - result: passed; Nextest run id `5c85e308-22d9-4ab5-8d94-3ca48ccf3819`.
- bootstrap predicate evaluation:
  `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics paint_cache_hit_test_only_replay_predicates_count_ring_snapshot_maxes --no-fail-fast --no-capture`
  - result: passed; Nextest run id `34eb1b6e-b6f7-4d06-80b0-ea1b1c6e764a`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\diag\ui-gallery-hit-test-only-paint-cache-probe-sweep.json --dir target\fret-diag-hit-test-only-paint-cache-probe-sweep-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779247865248`.
- suite runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-hit-test-only-paint-cache --dir target\fret-diag-hit-test-only-paint-cache-suite-v3 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779249205997`.

## Moved Cache-Root Root-Only Hit Path Gate

- invariant:
  a root-only `hit_test_path_cache` entry from the previous pointer position must not hide a clean
  view-cache child that moves under the same pointer while prepaint reuses translated interaction
  records.
- finding:
  no stale hit-routing defect was reproduced. The focused guard shows the existing cached-path
  fallback rejects a root-only cached path when the terminal node still has hit-testable children,
  then performs a full hit test and returns the moved leaf.
- implementation anchors:
  `crates/fret-ui/src/tree/tests/prepaint.rs`,
  `crates/fret-ui/src/tree/hit_test.rs`, and
  `crates/fret-ui/src/tree/prepaint/interaction.rs`.
- workstream/static checks:
  `python -m json.tool docs\workstreams\fret-mechanism-harness-v1\WORKSTREAM.json > $null`;
  `python tools\check_workstream_catalog.py`;
  `git diff --check`
  - result: passed.
  - note: `check_workstream_catalog.py` required refreshing the global
    `docs/workstreams/README.md` dedicated-directory count/date from 427 to 428 / 2026-05-20.
- format:
  `rustfmt --edition 2024 --check crates\fret-ui\src\tree\tests\prepaint.rs`
  - result: passed.
- focused mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui prepaint_interaction_cache_root_move_invalidates_stale_root_only_hit_path prepaint_interaction_cache_replay_translates_records_when_cache_root_moves --no-fail-fast --no-capture`
  - result: passed; Nextest run id `84167c6c-c03b-4feb-aa11-0693f55659b2`.
  - note: the run emitted the pre-existing `current_effective_opacity` dead-code warning in
    `crates\fret-ui\src\elements\runtime.rs`; this slice did not touch that file.

## Hit-Test Path Cache Higher-Z Sibling Gate

- invariant:
  a cached `root -> lower_child` hit path must not remain valid when a higher-z sibling moves under
  the same pointer. The cached-path fast path must reject the stale lower-child path before runtime
  routing can diverge from fallback hit testing.
- finding:
  no stale z-order routing defect was reproduced. The focused guard shows the existing sibling
  scan rejects the stale child path, fallback hit testing accepts the moved higher-z sibling, and
  the fallback result refreshes the path cache for subsequent reuse.
- implementation anchors:
  `crates/fret-ui/src/tree/tests/hit_test.rs` and
  `crates/fret-ui/src/tree/hit_test.rs`.
- format:
  `rustfmt --edition 2024 --check crates\fret-ui\src\tree\tests\hit_test.rs`
  - result: passed.
- focused mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer hit_test_layers_cached_reuses_path_and_respects_layer_order --no-fail-fast --no-capture`
  - result: passed; Nextest run id `92315d8d-56fd-4c3e-bfc1-bbfc849e954b`.
  - note: the run emitted the pre-existing `current_effective_opacity` dead-code warning in
    `crates\fret-ui\src\elements\runtime.rs`; this slice did not touch that file.

## Pointer-Move Dispatch Stale Hit-Path Gate

- invariant:
  pointer-move dispatch must not deliver events through a stale cached `root -> lower_child` path
  after a higher-z sibling moves under the same pointer. The real dispatch path must reject stale
  path-cache reuse before building the mapped event chain.
- finding:
  no stale pointer-move dispatch defect was reproduced. The focused dispatch guard shows
  `UiTree::dispatch_event` routes the first move to the lower child, rejects that stale path after a
  higher-z sibling moves under the same pointer, delivers the second move to the moved sibling, and
  then records a cache hit for the refreshed higher-z path on a third move.
- implementation anchors:
  `crates/fret-ui/src/tree/tests/hit_test.rs`,
  `crates/fret-ui/src/tree/hit_test.rs`, and
  `crates/fret-ui/src/tree/dispatch/window.rs`.
- workstream/static checks:
  `python -m json.tool docs\workstreams\fret-mechanism-harness-v1\WORKSTREAM.json > $null`;
  `python tools\check_workstream_catalog.py`;
  `git diff --check`
  - result: passed.
- format:
  `rustfmt --edition 2024 --check crates\fret-ui\src\tree\tests\hit_test.rs`
  - result: passed.
- focused mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui pointer_move_dispatch_rejects_stale_path_when_higher_z_sibling_moves_under_pointer hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer --no-fail-fast --no-capture`
  - result: passed; Nextest run id `093b8a5d-e67a-4b35-ab82-e02389f63173`.
  - note: the run emitted the pre-existing `current_effective_opacity` dead-code warning in
    `crates\fret-ui\src\elements\runtime.rs`; this slice did not touch that file.


## Hit-Test Path-Cache Runtime Hit Counter Gate

- invariant:
  UI Gallery pointer sweeps over a stable HitTestOnly/paint-cache surface must exercise the
  cached hit-test path fast path when bounds-tree queries are disabled. The runtime gate should
  prove both paint-cache replay and `hit_test_path_cache_hits` from debug snapshot history.
- finding:
  the strict gate initially failed: large-ring bundles showed paint-cache replay was allowed, but
  path-cache hits stayed at zero while misses reached two. The pointer and hit path were correct;
  the mechanism was over-conservative. Cached-path sibling validation rejected reuse whenever a
  higher-z sibling used transforms or non-clipping hit-test policy, even when full hit testing for
  that sibling returned no hit. The fix validates siblings with real hit-test semantics.
- implementation anchors:
  `crates/fret-ui/src/tree/hit_test.rs`,
  `crates/fret-ui/src/tree/tests/hit_test.rs`,
  `crates/fret-diag-protocol/src/lib.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
  `tools/diag-scripts/ui-gallery/diag/ui-gallery-hit-test-only-paint-cache-probe-sweep.json`, and
  `apps/fret-ui-gallery/src/ui/snippets/ai/prompt_input_cursor_demo.rs`.
- evidence anchors:
  strict-predicate failing bundle before the mechanism fix:
  `target/fret-diag-hit-test-only-paint-cache-path-cache-debug2/sessions/1779257690739-143660/1779257705567/bundle.schema2.json`;
  passing focused runtime AI packet:
  `target/fret-diag-hit-test-only-paint-cache-path-cache-v2/sessions/1779259321228-145468/1779259408910/ai.packet`;
  passing focused runtime pack:
  `target/fret-diag-hit-test-only-paint-cache-path-cache-v2/sessions/1779259321228-145468/share/1779259408910.zip`;
  passing suite summary:
  `target/fret-diag-hit-test-only-paint-cache-suite-path-cache-v2/sessions/1779259631852-148980/suite.summary.json`.
- format:
  `rustfmt --edition 2024 --check crates\fret-ui\src\tree\hit_test.rs crates\fret-ui\src\tree\tests\hit_test.rs`
  - result: passed.
- focused mechanism regression:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui hit_test_layers_cached_ignores_non_hit_testable_overlapping_higher_z_siblings hit_test_layers_cached_checks_transformed_higher_z_siblings_before_reuse hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer pointer_move_dispatch_rejects_stale_path_when_higher_z_sibling_moves_under_pointer --no-fail-fast --no-capture`
  - result: passed; Nextest run id `3d58d069-af7b-4675-a455-9f6ace214151`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- bootstrap predicate evaluation:
  `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics hit_test_path_cache_predicates_count_ring_snapshot_maxes paint_cache_hit_test_only_replay_predicates_count_ring_snapshot_maxes --no-fail-fast --no-capture`
  - result: passed; Nextest run id `affc9fa2-0c6d-47e8-b252-ae83c14e9059`.
- protocol roundtrip and predicate serialization:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_hit_test_path_cache_counters_serialize script_v2_roundtrip_ui_gallery_hit_test_only_paint_cache_probe_sweep --no-fail-fast --no-capture`
  - result: passed; Nextest run id `f0505399-96a7-4a92-95d7-398b48b1fd96`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\diag\ui-gallery-hit-test-only-paint-cache-probe-sweep.json --dir target\fret-diag-hit-test-only-paint-cache-path-cache-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779259408910`.
- runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-hit-test-only-paint-cache --dir target\fret-diag-hit-test-only-paint-cache-suite-path-cache-v2 --session-auto --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; script run id `1779259645062`; summary
    `target/fret-diag-hit-test-only-paint-cache-suite-path-cache-v2/sessions/1779259631852-148980/suite.summary.json`.

## RadioGroup Checked-State Mutation Runtime Gate

- invariant:
  RadioGroup item `checked` semantics must move with real user selection. A visual dot and focus
  ring are insufficient: diagnostics must prove the semantics nodes for Free, Pro, and Enterprise
  update their `checked` flags after pointer activation.
- finding:
  no RadioGroup recipe/runtime defect was reproduced. The gap was missing non-list RadioGroup
  checked-state mutation coverage; existing RadioGroup diagnostics covered label focus and RTL or
  choice-card layout without asserting dynamic `checked` semantics. The broader
  `ui-gallery-shadcn-runtime-evidence` suite currently has an unrelated Command
  retained-active-descendant `script_stalled_no_frames` failure before it reaches RadioGroup, so the
  durable gate is a focused `ui-gallery-radio-group-semantics` suite.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-checked-state-mutation.json`,
  `tools/diag-scripts/suites/ui-gallery-radio-group-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-radio-group-checked-state-mutation-v1/sessions/1779261539435-153996/1779261557779/ai.packet`;
  focused runtime pack:
  `target/fret-diag-radio-group-checked-state-mutation-v1/sessions/1779261539435-153996/share/1779261557779.zip`;
  dedicated suite summary:
  `target/fret-diag-radio-group-semantics-suite-v4/sessions/1779263168285-151724/suite.summary.json`;
  unrelated broad-suite Command no-frame failure:
  `target/fret-diag-shadcn-runtime-evidence-radio-group-checked-v1/sessions/1779261599311-79120/suite.summary.json`.
- JSON/registry:
  `python -m json.tool tools\diag-scripts\ui-gallery\radio-group\ui-gallery-radio-group-checked-state-mutation.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-radio-group-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- format:
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_radio_group_checked_state_mutation --no-fail-fast --no-capture`
  - result: passed; latest Nextest run id `7ced9cb1-5ecc-43bd-b118-4fc3cd0c6681`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\radio-group\ui-gallery-radio-group-checked-state-mutation.json --dir target\fret-diag-radio-group-checked-state-mutation-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779261557779`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-radio-group-semantics --dir target\fret-diag-radio-group-semantics-suite-v4 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; script run id `1779263181042`.
- broad runtime suite triage:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-radio-group-checked-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: failed before reaching RadioGroup because existing
    `ui-gallery-command-retained-active-descendant-action-state.json` stalled with no frames at
    step 1. A focused rerun of that Command script also failed with `script_stalled_no_frames`, so
    this is recorded as a separate diagnostics stability follow-up.

## Desktop Repeating-Timer Redraw Starvation Repair

- invariant:
  a repeating runner timer must not fire more than once in the same event-loop tick, and repeating
  timers must rearm from handler completion time rather than from the stale timestamp captured at
  the beginning of an effect-drain turn. Diagnostics keepalive timers may request redraw and inject
  events, but they must not starve the platform `RedrawRequested` they are trying to observe.
- finding:
  the broad `ui-gallery-shadcn-runtime-evidence` suite failure after the RadioGroup promotion was a
  real runner scheduling defect, not a Command recipe defect. The Command
  retained-active-descendant script stalled after `scroll_into_view` because a repeating
  diagnostics keepalive timer could catch up inside the same fixed-point drain turn. Once the
  runner tick guard and completion-time rearm landed, the focused Command script and the full broad
  suite passed.
- implementation anchors:
  `crates/fret-launch/src/runner/desktop/runner/timers.rs`,
  `crates/fret-launch/src/runner/desktop/runner/window.rs`,
  `crates/fret-launch/src/runner/desktop/runner/asset_reload.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`, and
  `ecosystem/fret-bootstrap/src/ui_app_driver.rs`.
- evidence anchors:
  focused Command AI packet:
  `target/fret-diag-command-retained-active-descendant-action-state-runner-timer-fresh-20260521/sessions/1779298813208-173816/1779298834262/ai.packet`;
  focused Command pack:
  `target/fret-diag-command-retained-active-descendant-action-state-runner-timer-fresh-20260521/sessions/1779298813208-173816/share/1779298834262.zip`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-runner-timer-fresh-20260521/sessions/1779299075645-7824/suite.summary.json`.
- timer regression:
  `cargo test --profile dev-fast -p fret-launch repeating_timer --lib -- --nocapture`
  - result: passed; 2 tests.
- diagnostics no-frame regression:
  `cargo test --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics --lib no_frame_keepalive -- --nocapture`
  - result: passed; 3 tests.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\command\ui-gallery-command-retained-active-descendant-action-state.json --dir target\fret-diag-command-retained-active-descendant-action-state-runner-timer-fresh-20260521 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779298834262`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-runner-timer-fresh-20260521 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 13/13; `stage_counts={"passed":13}`; `reason_code_counts={}`.
  - Command retained-active-descendant row: passed, run id `1779299206755`.
  - RadioGroup checked-state mutation row: passed, run id `1779299568012`.

## Runner Repeating-Timer Overlap Stress Gate

- invariant:
  the same-tick repeating-timer guard must apply across overlapping timers. A window-targeted
  diagnostics keepalive timer and a windowless asset-reload-style polling timer can both be due in
  a stale drain turn, but neither may catch up more than once before the runner tick advances.
- finding:
  no new runtime defect was reproduced. This is a regression-hardening companion to the F242
  runtime failure so future scheduler changes can be checked cheaply without launching UI Gallery.
- implementation anchors:
  `crates/fret-launch/src/runner/desktop/runner/timers.rs`.
- focused stress gate:
  `cargo test --profile dev-fast -p fret-launch overlapping_repeating_timers --lib -- --nocapture`
  - result: passed; 1 test.
- full timer regression filter:
  `cargo test --profile dev-fast -p fret-launch repeating_timer --lib -- --nocapture`
  - result: passed; 3 tests.

## Switch Choice-Card Checked-State Runtime Gate

- invariant:
  card-style `FieldLabel::wrap(...)` activation on the shadcn Switch docs-path Choice Card must
  toggle the associated nested `Switch` and refresh the exported `checked` semantics for each
  independent control. Visual card chrome and associated label hit testing are not enough without
  proving the control semantics update.
- finding:
  no Switch recipe/runtime defect was reproduced. This slice closes missing coverage for a real
  non-list form composition where card label activation, `ControlId` association, model mutation,
  and exported `checked` semantics all have to stay aligned.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-choice-card-checked-state-mutation.json`,
  `tools/diag-scripts/suites/ui-gallery-switch-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-switch-choice-card-checked-state-mutation-v1/sessions/1779301451133-110692/1779301465865/ai.packet`;
  focused runtime pack:
  `target/fret-diag-switch-choice-card-checked-state-mutation-v1/sessions/1779301451133-110692/share/1779301465865.zip`;
  dedicated suite summary:
  `target/fret-diag-switch-semantics-suite-v1/sessions/1779301571842-137052/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-switch-choice-card-v1/sessions/1779301689107-175712/suite.summary.json`.
- JSON/registry:
  `python -m json.tool tools\diag-scripts\ui-gallery\switch\ui-gallery-switch-choice-card-checked-state-mutation.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-switch-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_switch_choice_card_checked_state_mutation --no-fail-fast --no-capture`
  - result: passed; Nextest run id `3d7a846c-ec57-4044-8222-e81a4b9f978f`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\switch\ui-gallery-switch-choice-card-checked-state-mutation.json --dir target\fret-diag-switch-choice-card-checked-state-mutation-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779301465865`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-switch-semantics --dir target\fret-diag-switch-semantics-suite-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; `stage_counts={"passed":1}`; script run id `1779301586186`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-switch-choice-card-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 14/14; `stage_counts={"passed":14}`; `reason_code_counts={}`;
    Switch choice-card row run id `1779302260684`.

## Checkbox Disabled Action-State Runtime Gate

- invariant:
  a disabled shadcn Checkbox must export `disabled=true`, suppress the `invoke` action, and keep its
  checked state unchanged through both direct control activation and associated label activation.
- finding:
  no Checkbox recipe/runtime defect was reproduced. This slice closes missing coverage for a real
  non-text form control where disabled chrome, `FieldLabel` control association, checked state, and
  exported action metadata all have to stay aligned.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-disabled-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-checkbox-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-checkbox-disabled-action-state-v1/sessions/1779303551841-64456/1779303569865/ai.packet`;
  focused runtime pack:
  `target/fret-diag-checkbox-disabled-action-state-v1/sessions/1779303551841-64456/share/1779303569865.zip`;
  dedicated suite summary:
  `target/fret-diag-checkbox-semantics-suite-v1/sessions/1779303815632-98108/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-checkbox-disabled-v1/sessions/1779304064247-171464/suite.summary.json`.
- JSON/registry:
  `python -m json.tool tools\diag-scripts\ui-gallery\checkbox\ui-gallery-checkbox-disabled-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-checkbox-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_checkbox_disabled_action_state --no-fail-fast --no-capture`
  - result: passed; Nextest run id `8ace631b-a63d-4a0d-bc9b-55ccc1a64267`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\checkbox\ui-gallery-checkbox-disabled-action-state.json --dir target\fret-diag-checkbox-disabled-action-state-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779303569865`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-checkbox-semantics --dir target\fret-diag-checkbox-semantics-suite-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; `stage_counts={"passed":1}`; script run id `1779303834251`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-checkbox-disabled-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 15/15; `stage_counts={"passed":15}`; `reason_code_counts={}`;
    Checkbox disabled-action row run id `1779304154355`.


## Slider Numeric Action-State and Thumb Test-ID Runtime Gate

- invariant:
  an enabled shadcn Slider thumb must export numeric value/min/max/step/jump plus enabled
  `set_value`, `increment`, and `decrement`; a disabled Slider thumb must keep numeric metadata but
  export `disabled=true` and suppress `set_value`, `increment`, `decrement`, and `focus`. Derived
  thumb automation ids must also stay unique across multi-thumb recipe chrome and semantic thumbs.
- finding:
  the new runtime script did not reproduce a Slider numeric/action semantics defect. The first
  linted suite run did find a real recipe diagnostics defect: visual thumb chrome reused bare
  `{prefix}-thumb` ids for every thumb, creating duplicate `test_id`s in multi-thumb Slider
  examples. The script also needed focus hygiene after scrolling away from a focused single thumb.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/slider.rs`,
  `ecosystem/fret-ui-shadcn/src/test_id.rs`,
  `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-numeric-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-slider-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- discovery evidence:
  initial lint failure before repair:
  `target/fret-diag-slider-semantics-suite-v1/sessions/1779306432063-174216/1779306461660-ui-gallery-slider-numeric-action-state/check.lint.json`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-slider-numeric-action-state-v2/sessions/1779307920873-121240/1779307931755/ai.packet`;
  focused runtime pack:
  `target/fret-diag-slider-numeric-action-state-v2/sessions/1779307920873-121240/share/1779307931755.zip`;
  dedicated suite summary:
  `target/fret-diag-slider-semantics-suite-v2/sessions/1779307963346-175080/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-slider-numeric-v1/sessions/1779308003840-152848/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\slider\ui-gallery-slider-numeric-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-slider-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check ecosystem\fret-ui-shadcn\src\test_id.rs ecosystem\fret-ui-shadcn\src\slider.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- Slider recipe unit gates:
  `cargo test --profile dev-fast -p fret-ui-shadcn multi_thumb_slider_derives_unique_thumb_test_ids --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-shadcn slider_set_value_numeric_updates_model_via_accessibility_driver --lib -- --nocapture`
  - result: passed; 1 test.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_slider_numeric_action_state --no-fail-fast --no-capture`
  - result: passed; Nextest run id `1e5e6033-7824-4d00-88c5-f9a857b3e4a8`.
- build:
  `cargo build --profile dev-fast -p fret-ui-gallery`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\slider\ui-gallery-slider-numeric-action-state.json --dir target\fret-diag-slider-numeric-action-state-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779307931755`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-slider-semantics --dir target\fret-diag-slider-semantics-suite-v2 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; `stage_counts={"passed":1}`; script run id `1779307974845`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-slider-numeric-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 16/16; `stage_counts={"passed":16}`; `reason_code_counts={}`;
    Slider numeric-action row run id `1779308615620`.


## Checkbox Table Mixed Checked-State Runtime Gate

- invariant:
  a shadcn Checkbox table select-all control must expose explicit tri-state checked semantics:
  `mixed` when only some rows are selected and `true` when all rows are selected. The same control
  must keep `invoke=true` while row mutations move it between mixed and checked states.
- finding:
  no Checkbox recipe/runtime defect was reproduced. The slice closed a harness/protocol gap by
  making explicit tri-state `checked_state` queryable instead of relying on the legacy binary
  `checked` flag or `checked_is_none`.
- implementation anchors:
  `crates/fret-diag-protocol/src/lib.rs`,
  `crates/fret-diag-protocol/src/builder.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`,
  `crates/fret-mechanism-harness/src/oracle.rs`,
  `crates/fret-mechanism-harness/src/lib.rs`,
  `docs/ui-diagnostics-and-scripted-tests.md`,
  `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-table-mixed-state-action.json`,
  `tools/diag-scripts/suites/ui-gallery-checkbox-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-checkbox-table-mixed-state-action-v1/sessions/1779310480442-177764/1779310495372/ai.packet`;
  focused runtime pack:
  `target/fret-diag-checkbox-table-mixed-state-action-v1/sessions/1779310480442-177764/share/1779310495372.zip`;
  dedicated suite summary:
  `target/fret-diag-checkbox-semantics-suite-table-mixed-v1/sessions/1779310724199-166384/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-checkbox-table-mixed-v1/sessions/1779311169346-151568/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\checkbox\ui-gallery-checkbox-table-mixed-state-action.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-checkbox-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\src\builder.rs crates\fret-diag-protocol\src\lib.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs crates\fret-mechanism-harness\src\lib.rs crates\fret-mechanism-harness\src\oracle.rs ecosystem\fret-bootstrap\src\ui_diagnostics\predicates.rs ecosystem\fret-bootstrap\src\ui_diagnostics\script_steps_wait.rs`;
  `git diff --check`
  - result: passed.
- protocol/bootstrap/mechanism gates:
  `cargo test --profile dev-fast -p fret-diag-protocol predicate_checked_state_is_serializes_and_deserializes --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics checked_state_is_matches_semantics_checked_state --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-mechanism-harness semantics_value_state_actions_and_structured_metadata_are_queryable --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui mechanism_harness_semantics_relations_match_oracles --lib -- --nocapture`
  - result: passed; 1 test.
- protocol script roundtrip:
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_checkbox_table_mixed_state_action -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\checkbox\ui-gallery-checkbox-table-mixed-state-action.json --dir target\fret-diag-checkbox-table-mixed-state-action-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779310495372`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-checkbox-semantics --dir target\fret-diag-checkbox-semantics-suite-table-mixed-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 2/2; `stage_counts={"passed":2}`; script run id `1779310910113`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-checkbox-table-mixed-v1 --session-auto --timeout-ms 900000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 17/17; `stage_counts={"passed":17}`; `reason_code_counts={}`;
    Checkbox table mixed-state row run id `1779311405413`.

## Toggle Pressed-State Runtime Gate

- invariant:
  a shadcn Toggle must expose explicit tri-state pressed semantics through `pressed_state`, not
  selected semantics. The Bookmark toggle should move `false -> true -> false` across two
  activations, keep `selected=false`, and keep `invoke=true`.
- finding:
  no Toggle recipe/runtime defect was reproduced. The existing script was wrong: after click, the
  runtime bundle showed `role=button` and `flags.pressed_state="true"`, while the script asserted
  `selected_is=true`. This slice closes that diagnostics expressiveness gap with a first-class
  `pressed_state_is` predicate and updates the Toggle gate to assert the correct semantics axis.
- implementation anchors:
  `crates/fret-diag-protocol/src/lib.rs`,
  `crates/fret-diag-protocol/src/builder.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`,
  `crates/fret-mechanism-harness/src/oracle.rs`,
  `crates/fret-mechanism-harness/src/lib.rs`,
  `docs/ui-diagnostics-and-scripted-tests.md`,
  `tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-interaction-screenshots.json`,
  `tools/diag-scripts/suites/ui-gallery-toggle-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- discovery evidence:
  old `selected_is=true` script failure:
  `target/fret-diag-toggle-interaction-selected-probe-v1/sessions/1779313631621-83488/1779313643019/ai.packet`;
  failure pack:
  `target/fret-diag-toggle-interaction-selected-probe-v1/sessions/1779313631621-83488/share/1779313643019.zip`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-toggle-pressed-state-interaction-v1/sessions/1779314794606-180768/1779314805300/ai.packet`;
  focused runtime pack:
  `target/fret-diag-toggle-pressed-state-interaction-v1/sessions/1779314794606-180768/share/1779314805300.zip`;
  dedicated suite summary:
  `target/fret-diag-toggle-semantics-suite-v1/sessions/1779314830681-87028/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-toggle-pressed-v2/sessions/1779316094427-64032/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\toggle\ui-gallery-toggle-interaction-screenshots.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-toggle-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\src\builder.rs crates\fret-diag-protocol\src\lib.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs crates\fret-mechanism-harness\src\lib.rs crates\fret-mechanism-harness\src\oracle.rs ecosystem\fret-bootstrap\src\ui_diagnostics\predicates.rs ecosystem\fret-bootstrap\src\ui_diagnostics\script_steps_wait.rs`;
  `git diff --check`
  - result: passed.
- protocol/bootstrap/mechanism gates:
  `cargo test --profile dev-fast -p fret-diag-protocol predicate_pressed_state_is_serializes_and_deserializes --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics pressed_state_is_matches_semantics_pressed_state --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-mechanism-harness semantics_value_state_actions_and_structured_metadata_are_queryable --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui mechanism_harness_semantics_relations_match_oracles --lib -- --nocapture`
  - result: passed; 1 test.
- protocol script roundtrip:
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_toggle_interaction_screenshots -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\toggle\ui-gallery-toggle-interaction-screenshots.json --dir target\fret-diag-toggle-pressed-state-interaction-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779314805300`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-toggle-semantics --dir target\fret-diag-toggle-semantics-suite-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; script run id `1779314840880`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-toggle-pressed-v2 --session-auto --timeout-ms 1200000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 18/18; `stage_counts={"passed":18}`; `reason_code_counts={}`;
    Toggle pressed-state row run id `1779317023787`.


## Input Required/Invalid Form-State Runtime Gate

- invariant:
  a shadcn Input must expose required and invalid semantics on the concrete TextInput control, not
  only through Field chrome. Invalid examples should export `invalid=true` and `required=false`;
  required examples should export `required=true` and no invalid state. Both controls remain enabled
  and must keep `focus=true` and `set_value=true`.
- finding:
  no Input recipe/runtime defect was reproduced. The slice closed a diagnostics expressiveness gap
  (`required_is` / `invalid_is`) and a UI Gallery automation surface gap: the Invalid and Required
  snippets now stamp stable concrete TextInput test ids so gates can target the owning node.
- implementation anchors:
  `crates/fret-diag-protocol/src/lib.rs`,
  `crates/fret-diag-protocol/src/builder.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`,
  `crates/fret-mechanism-harness/src/observe.rs`,
  `crates/fret-mechanism-harness/src/oracle.rs`,
  `crates/fret-mechanism-harness/src/lib.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/input/invalid.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/input/required.rs`,
  `docs/ui-diagnostics-and-scripted-tests.md`,
  `tools/diag-scripts/ui-gallery/input/ui-gallery-input-required-invalid-semantics.json`,
  `tools/diag-scripts/suites/ui-gallery-input-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-input-required-invalid-semantics-v2/sessions/1779318958257-166004/1779318973423/ai.packet`;
  focused runtime pack:
  `target/fret-diag-input-required-invalid-semantics-v2/sessions/1779318958257-166004/share/1779318973423.zip`;
  dedicated suite summary:
  `target/fret-diag-input-semantics-suite-v1/sessions/1779319043334-48440/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-input-required-invalid-v1/sessions/1779319155073-96032/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\input\ui-gallery-input-required-invalid-semantics.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-input-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\input\invalid.rs apps\fret-ui-gallery\src\ui\snippets\input\required.rs crates\fret-diag-protocol\src\builder.rs crates\fret-diag-protocol\src\lib.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs crates\fret-mechanism-harness\src\lib.rs crates\fret-mechanism-harness\src\observe.rs crates\fret-mechanism-harness\src\oracle.rs ecosystem\fret-bootstrap\src\ui_diagnostics\predicates.rs ecosystem\fret-bootstrap\src\ui_diagnostics\script_steps_wait.rs`;
  `git diff --check`
  - result: passed.
- protocol/bootstrap/mechanism gates:
  `cargo test --profile dev-fast -p fret-diag-protocol predicate_required_is_serializes_and_deserializes --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-diag-protocol predicate_invalid_is_serializes_and_deserializes --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics required_and_invalid_is_match_form_control_semantics_flags --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-mechanism-harness semantics_value_state_actions_and_structured_metadata_are_queryable --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui mechanism_harness_semantics_relations_match_oracles --lib -- --nocapture`
  - result: passed; 1 test.
- protocol script roundtrip:
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_input_required_invalid_semantics -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-required-invalid-semantics.json --dir target\fret-diag-input-required-invalid-semantics-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779318973423`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-input-semantics --dir target\fret-diag-input-semantics-suite-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 2/2; `stage_counts={"passed":2}`; script run id `1779319084263`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-input-required-invalid-v1 --session-auto --timeout-ms 1200000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 19/19; `stage_counts={"passed":19}`; `reason_code_counts={}`;
    Input required/invalid row run id `1779319975211`.


## Select Invalid Form-State Runtime Gate

- invariant:
  a shadcn Select Invalid example must expose invalid form-state semantics on the concrete trigger
  combobox while no value is selected, not only through Field chrome. After committing a value, the
  trigger should clear invalid semantics, the FieldError should disappear, and the trigger should
  retain enabled focus/invoke actions.
- finding:
  no Select recipe/runtime defect was reproduced. The slice promoted existing Select semantics
  behavior into a live docs-path runtime gate and into the broad shadcn runtime-evidence suite.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/select/ui-gallery-select-invalid-form-state.json`,
  `tools/diag-scripts/suites/ui-gallery-select-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- existing recipe anchors:
  `ecosystem/fret-ui-shadcn/src/select.rs` (`Select::aria_invalid`, `Select::required`, and the
  existing unit tests `select_aria_invalid_exposes_invalid_semantics` /
  `select_required_exposes_required_semantics`).
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-select-invalid-form-state-v1/sessions/1779321642042-159492/1779321650994/ai.packet`;
  focused runtime pack:
  `target/fret-diag-select-invalid-form-state-v1/sessions/1779321642042-159492/share/1779321650994.zip`;
  dedicated suite summary:
  `target/fret-diag-select-semantics-suite-v1/sessions/1779321672604-100084/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-select-invalid-v1/sessions/1779321710285-137352/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\select\ui-gallery-select-invalid-form-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-select-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- Select recipe semantics gates:
  `cargo test --profile dev-fast -p fret-ui-shadcn select_aria_invalid_exposes_invalid_semantics --lib -- --nocapture`
  - result: passed; also matched `native_select::tests::native_select_aria_invalid_exposes_invalid_semantics`.
  `cargo test --profile dev-fast -p fret-ui-shadcn select_required_exposes_required_semantics --lib -- --nocapture`
  - result: passed; also matched `native_select::tests::native_select_required_exposes_required_semantics`.
- protocol script roundtrip:
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_select_invalid_form_state -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\select\ui-gallery-select-invalid-form-state.json --dir target\fret-diag-select-invalid-form-state-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779321650994`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-select-semantics --dir target\fret-diag-select-semantics-suite-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; script run id `1779321682202`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-select-invalid-v1 --session-auto --timeout-ms 1200000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 20/20; `stage_counts={"passed":20}`; `reason_code_counts={}`;
    Select invalid row run id `1779322696285`.

## Textarea Required/Invalid Form-State Runtime Gate

- invariant:
  a shadcn Textarea must expose required and invalid semantics on the concrete TextArea control,
  not only through surrounding Field chrome. Invalid examples should export `invalid=true` and
  `required=false`; required examples should export `required=true` and no invalid state. Both
  controls remain enabled and must keep `focus=true` and `set_value=true`.
- finding:
  no Textarea recipe/runtime defect was reproduced. The slice closed a UI Gallery automation surface
  gap: the Invalid snippet now stamps a stable concrete TextArea test id, and the docs page now has
  a Required example with caller-owned marker composition and control-owned `required` semantics.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/textarea/invalid.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/textarea/required.rs`,
  `apps/fret-ui-gallery/src/ui/pages/textarea.rs`,
  `ecosystem/fret-ui-shadcn/src/textarea.rs`,
  `tools/diag-scripts/ui-gallery/textarea/ui-gallery-textarea-required-invalid-semantics.json`,
  `tools/diag-scripts/suites/ui-gallery-textarea-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-textarea-required-invalid-semantics-v1/sessions/1779324589606-162572/1779324602377/ai.packet`;
  focused runtime pack:
  `target/fret-diag-textarea-required-invalid-semantics-v1/sessions/1779324589606-162572/share/1779324602377.zip`;
  dedicated suite summary:
  `target/fret-diag-textarea-semantics-suite-v1/sessions/1779324642363-184708/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-textarea-required-invalid-v2/sessions/1779326355415-96352/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\textarea\ui-gallery-textarea-required-invalid-semantics.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery\textarea\ui-gallery-textarea-docs-screenshot.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-textarea-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\textarea\invalid.rs apps\fret-ui-gallery\src\ui\snippets\textarea\required.rs apps\fret-ui-gallery\src\ui\snippets\textarea\mod.rs apps\fret-ui-gallery\src\ui\pages\textarea.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs ecosystem\fret-ui-shadcn\src\textarea.rs apps\fret-ui-gallery\tests\textarea_docs_surface.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui-shadcn textarea_required_builder_sets_textarea_required_semantics --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-shadcn textarea_aria_invalid_builder_sets_textarea_invalid_semantics --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_textarea_required_invalid_semantics -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test textarea_docs_surface textarea_page_documents_source_axes_and_leaf_children_api_decision -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test textarea_docs_surface textarea_snippets_keep_the_docs_path_examples_and_leaf_surface -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test textarea_docs_surface textarea_diag_scripts_cover_docs_path_and_label_follow_up -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app textarea_snippets_prefer_ui_cx_on_the_default_app_surface -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app textarea_page_uses_typed_doc_sections_for_app_facing_snippets -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app checkbox_radio_input_and_textarea_docs_keep_required_ownership_on_the_control_surface -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app checkbox_radio_input_and_textarea_docs_keep_invalid_ownership_on_the_control_surface -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\textarea\ui-gallery-textarea-required-invalid-semantics.json --dir target\fret-diag-textarea-required-invalid-semantics-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779324602377`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-textarea-semantics --dir target\fret-diag-textarea-semantics-suite-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; script run id `1779324654768`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-textarea-required-invalid-v2 --session-auto --timeout-ms 1800000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 21/21; `stage_counts={"passed":21}`; `reason_code_counts={}`;
    Textarea row run id `1779327669604`.

## InputOTP Invalid/Required Form-State Runtime Gate

- invariant:
  shadcn InputOTP slot-invalid visual chrome must promote invalid form-state semantics to the
  hidden root OTP TextInput, because that node owns editing, value, focus, and accessibility. The
  Form example must expose required semantics on the same hidden root control. Both controls remain
  enabled and keep `focus=true` and `set_value=true`.
- finding:
  no InputOTP recipe/runtime defect was reproduced. Existing recipe behavior was correct; the slice
  promoted it into a deterministic runtime gate and avoided long-page suite drift by scoping the
  page to `Invalid,Form`.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/input/ui-gallery-input-otp-invalid-required-semantics.json`,
  `tools/diag-scripts/ui-gallery-input-otp-invalid-required-semantics.json`,
  `tools/diag-scripts/suites/ui-gallery-input-otp-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- existing recipe anchors:
  `ecosystem/fret-ui-shadcn/src/input_otp.rs`
  (`InputOtp::required`, `InputOtpSlot::aria_invalid`, and the existing unit tests
  `input_otp_slot_part_aria_invalid_sets_hidden_input_semantics_invalid` /
  `input_otp_required_builder_exposes_required_semantics`).
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-input-otp-invalid-required-semantics-v2/sessions/1779329862109-172384/1779329877911/ai.packet`;
  focused runtime pack:
  `target/fret-diag-input-otp-invalid-required-semantics-v2/sessions/1779329862109-172384/share/1779329877911.zip`;
  dedicated suite summary:
  `target/fret-diag-input-otp-semantics-suite-v2/sessions/1779329954569-181280/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-input-otp-invalid-required-v1/sessions/1779330056274-195352/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\input\ui-gallery-input-otp-invalid-required-semantics.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-input-otp-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-input-otp-invalid-required-semantics.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui-shadcn input_otp_slot_part_aria_invalid_sets_hidden_input_semantics_invalid --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-shadcn input_otp_required_builder_exposes_required_semantics --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_input_otp_invalid_required_semantics -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-otp-invalid-required-semantics.json --dir target\fret-diag-input-otp-invalid-required-semantics-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779329877911`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-input-otp-semantics --dir target\fret-diag-input-otp-semantics-suite-v2 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; script run id `1779329969582`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-input-otp-invalid-required-v1 --session-auto --timeout-ms 1800000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 22/22; `stage_counts={"passed":22}`; `reason_code_counts={}`;
    InputOTP row run id `1779331449795`.

## DatePicker Required/Invalid Trigger Semantics Runtime Gate

- invariant:
  shadcn DatePicker is a button-backed popover trigger, so required and invalid form-state semantics
  must be exported by the trigger button that owns focus/invoke and opens the calendar. Surrounding
  `Field::invalid(true)` styling and `FieldError` copy remain caller-owned.
- finding:
  no DatePicker recipe/runtime defect was reproduced. Existing trigger semantics were correct; this
  slice closed the missing UI Gallery invalid teaching surface and promoted the required/invalid
  trigger contract into a deterministic runtime gate. The first broad-suite run also exposed a
  diagnostics authoring hazard in the existing Select invalid gate: `click_stable` on an already
  visible transient overlay option could stall with `timeout.no_frames`. The Select script now uses
  direct semantic `click` for that committed option.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/date_picker/invalid.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/date_picker/mod.rs`,
  `apps/fret-ui-gallery/src/ui/pages/date_picker.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-required-invalid-semantics.json`,
  `tools/diag-scripts/ui-gallery-date-picker-required-invalid-semantics.json`,
  `tools/diag-scripts/suites/ui-gallery-date-picker-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/ui-gallery/select/ui-gallery-select-invalid-form-state.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- existing recipe anchors:
  `ecosystem/fret-ui-shadcn/src/date_picker.rs`
  (`DatePicker::required`, `DatePicker::aria_invalid`, and the existing unit tests
  `date_picker_required_exposes_required_semantics` /
  `date_picker_aria_invalid_exposes_invalid_semantics`).
- evidence anchors:
  focused DatePicker runtime AI packet:
  `target/fret-diag-date-picker-required-invalid-semantics-v1/sessions/1779334955994-144056/1779334968449/ai.packet`;
  focused DatePicker runtime pack:
  `target/fret-diag-date-picker-required-invalid-semantics-v1/sessions/1779334955994-144056/share/1779334968449.zip`;
  dedicated DatePicker suite summary:
  `target/fret-diag-date-picker-semantics-suite-v1/sessions/1779335003408-192508/suite.summary.json`;
  hardened Select focused runtime AI packet:
  `target/fret-diag-select-invalid-form-state-click-hardening-v1/sessions/1779337219903-189420/1779337229373/ai.packet`;
  hardened Select focused runtime pack:
  `target/fret-diag-select-invalid-form-state-click-hardening-v1/sessions/1779337219903-189420/share/1779337229373.zip`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-date-picker-required-invalid-v2/sessions/1779337267974-91608/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\date-picker\ui-gallery-date-picker-required-invalid-semantics.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery\select\ui-gallery-select-invalid-form-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-date-picker-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-date-picker-required-invalid-semantics.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\date_picker\invalid.rs apps\fret-ui-gallery\src\ui\snippets\date_picker\mod.rs apps\fret-ui-gallery\src\ui\pages\date_picker.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui-shadcn date_picker_required_exposes_required_semantics --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-shadcn date_picker_aria_invalid_exposes_invalid_semantics --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_date_picker_required_invalid_semantics -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app date_picker_and_input_otp_docs_keep_required_ownership_on_the_control_surface -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app date_picker_docs_keep_invalid_ownership_on_trigger_with_caller_owned_error_copy -- --nocapture`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_select_invalid_form_state -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused DatePicker runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\date-picker\ui-gallery-date-picker-required-invalid-semantics.json --dir target\fret-diag-date-picker-required-invalid-semantics-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779334968449`.
- hardened Select focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\select\ui-gallery-select-invalid-form-state.json --dir target\fret-diag-select-invalid-form-state-click-hardening-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779337229373`.
- dedicated DatePicker runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-date-picker-semantics --dir target\fret-diag-date-picker-semantics-suite-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; script run id `1779335013570`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-date-picker-required-invalid-v2 --session-auto --timeout-ms 2400000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 23/23; `stage_counts={"passed":23}`; `reason_code_counts={}`;
    DatePicker row run id `1779338165138`; hardened Select row run id `1779338502088`.

## Form Submit Validation Semantics Runtime Gate

- invariant:
  submit-driven `FormState` validation must mutate the semantics on the concrete controls decorated
  by `FormField`, not only the surrounding visual field chrome. Required semantics are present
  before submit; invalid semantics and caller-owned alert copy appear only after submit; on-change
  repair clears invalid state; a final valid submit reports success.
- finding:
  no Form/FormField recipe or runtime defect was reproduced. Existing form registration,
  submit-validation, and on-change revalidation behavior was correct; this slice promoted the
  user-level multi-control validation journey into a deterministic runtime gate.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/form/submit_validation.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/form/mod.rs`,
  `apps/fret-ui-gallery/src/ui/pages/form.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `tools/diag-scripts/ui-gallery/form/ui-gallery-form-submit-validation-semantics.json`,
  `tools/diag-scripts/ui-gallery-form-submit-validation-semantics.json`,
  `tools/diag-scripts/suites/ui-gallery-form-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- existing recipe anchors:
  `ecosystem/fret-ui-shadcn/src/form_field.rs`,
  `ecosystem/fret-ui-kit/src/declarative/form.rs`, and
  `ecosystem/fret-ui-headless/src/form_state.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-form-submit-validation-semantics-v1/sessions/1779342775326-181112/1779342789863/ai.packet`;
  focused runtime pack:
  `target/fret-diag-form-submit-validation-semantics-v1/sessions/1779342775326-181112/share/1779342789863.zip`;
  dedicated suite summary:
  `target/fret-diag-form-semantics-suite-v1/sessions/1779342834313-22100/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-form-submit-validation-v2/sessions/1779344474432-184028/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\form\ui-gallery-form-submit-validation-semantics.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-form-submit-validation-semantics.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-form-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py --write`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\form\submit_validation.rs apps\fret-ui-gallery\src\ui\snippets\form\mod.rs apps\fret-ui-gallery\src\ui\pages\form.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui-shadcn form_field_ --lib -- --nocapture`
  - result: passed; 29 tests.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app form_ -- --nocapture`
  - result: passed; 8 tests.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_form_submit_validation_semantics -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
  - note: the run emitted the pre-existing unrelated unused `start` warning from
    `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\form\ui-gallery-form-submit-validation-semantics.json --dir target\fret-diag-form-submit-validation-semantics-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779342789863`.
- dedicated runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-form-semantics --dir target\fret-diag-form-semantics-suite-v1 --session-auto --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; script run id `1779342849187`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-form-submit-validation-v2 --session-auto --timeout-ms 2400000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 24/24; `stage_counts={"passed":24}`; `reason_code_counts={}`;
    Form row run id `1779345780709`.

## Form Disabled Field Action-State Runtime Gate

- invariant:
  shadcn Field disabled styling is field-shell/group state. The concrete disabled control must own
  the accessibility and action semantics: `disabled=true`, `focus=false`, and `set_value=false`.
  Sibling controls must not become disabled merely because a nearby Field shell is disabled.
- finding:
  no Form/Field recipe or runtime defect was reproduced. The gate corrected two diagnostics hazards
  during authoring: an over-specific visual opacity probe was removed because it did not prove the
  action-state invariant, and the companion value assertion now waits for the asynchronous model
  update after `set_text_value`.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/form/disabled_field.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/form/mod.rs`,
  `apps/fret-ui-gallery/src/ui/pages/form.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `tools/diag-scripts/ui-gallery/form/ui-gallery-form-disabled-field-action-state.json`,
  `tools/diag-scripts/ui-gallery-form-disabled-field-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-form-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  upstream `repo-ref/ui/apps/v4/registry/new-york-v4/ui/field.tsx` has `Field` as a `div` group with
  `data-disabled` styling; concrete controls own real disabled semantics.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-form-disabled-field-action-state-v3/sessions/1779351791667-101584/1779351805821/ai.packet`;
  focused runtime pack:
  `target/fret-diag-form-disabled-field-action-state-v3/sessions/1779351791667-101584/share/1779351805821.zip`;
  dedicated suite summary:
  `target/fret-diag-form-semantics-suite-disabled-field-rebuilt-v1/sessions/1779352801343-181708/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-form-disabled-field-v1/sessions/1779352876487-190332/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\form\ui-gallery-form-disabled-field-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-form-disabled-field-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-form-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\form\disabled_field.rs apps\fret-ui-gallery\src\ui\snippets\form\mod.rs apps\fret-ui-gallery\src\ui\pages\form.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app form_ -- --nocapture`
  - result: passed; 9 tests.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_form_disabled_field_action_state -- --nocapture`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
  - note: rebuilding with `gallery-dev` was required after an earlier targeted Gallery test rebuilt
    `target\dev-fast\fret-ui-gallery.exe` without the gallery feature and hid the Form page from
    runtime diagnostics.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\form\ui-gallery-form-disabled-field-action-state.json --dir target\fret-diag-form-disabled-field-action-state-v3 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779351805821`.
- dedicated Form runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-form-semantics\suite.json --dir target\fret-diag-form-semantics-suite-disabled-field-rebuilt-v1 --session-auto --timeout-ms 600000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 2/2; `stage_counts={"passed":2}`; disabled Field row run id `1779352815799`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-form-disabled-field-v1 --session-auto --timeout-ms 2400000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 25/25; `stage_counts={"passed":25}`; `reason_code_counts={}`;
    disabled Field row run id `1779354050679`.

## RadioGroup Required Disabled Action-State Runtime Gate

- invariant:
  shadcn/Radix RadioGroup owns required state on the group root, while disabled selection
  suppression belongs to concrete radio items. A disabled item and its associated
  `FieldLabel::for_control(...)` bridge must not mutate the selected value, but enabled sibling
  items must remain focusable/invokable.
- finding:
  the disabled action-state path behaved correctly, but the slice found a semantics completeness gap:
  RadioGroup items exposed legacy `checked` only. `radio_button_a11y(...)` now stamps explicit
  `checked_state=true|false` as well, and the shadcn semantics unit test asserts both channels.
- implementation anchors:
  `ecosystem/fret-ui-kit/src/primitives/radio_group.rs`,
  `ecosystem/fret-ui-shadcn/src/radio_group.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/radio_group/required_disabled.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/radio_group/mod.rs`,
  `apps/fret-ui-gallery/src/ui/pages/radio_group.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-required-disabled-action-state.json`,
  `tools/diag-scripts/ui-gallery-radio-group-required-disabled-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-radio-group-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  upstream `repo-ref/ui/apps/v4/registry/new-york-v4/ui/radio-group.tsx` keeps disabled item chrome
  on `RadioGroupPrimitive.Item`; Radix/APG owns the required group and radio checked-state semantics.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-radio-group-required-disabled-action-state-v1/sessions/1779358454419-182932/1779358468383/ai.packet`;
  focused runtime pack:
  `target/fret-diag-radio-group-required-disabled-action-state-v1/sessions/1779358454419-182932/share/1779358468383.zip`;
  dedicated suite summary:
  `target/fret-diag-radio-group-semantics-suite-required-disabled-v1/sessions/1779358495275-211416/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-radio-group-required-disabled-v1/sessions/1779358567107-208948/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\radio-group\ui-gallery-radio-group-required-disabled-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-radio-group-required-disabled-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-radio-group-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\index.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check ecosystem\fret-ui-kit\src\primitives\radio_group.rs ecosystem\fret-ui-shadcn\src\radio_group.rs apps\fret-ui-gallery\src\ui\snippets\radio_group\required_disabled.rs apps\fret-ui-gallery\src\ui\snippets\radio_group\mod.rs apps\fret-ui-gallery\src\ui\pages\radio_group.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui-shadcn radio_group_emits_radio_group_and_radio_button_semantics --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib radio_group_emits_radio_group_and_radio_button_semantics`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app radio_group_ -- --nocapture`
  - result: passed; 7 tests.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app radio_group_required_disabled_snippet_keeps_group_required_and_item_action_state_separate`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_radio_group_required_disabled_action_state -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_radio_group_required_disabled_action_state`
  - result: passed; 1 test.
  Note: an initial `cargo nextest run --profile dev-fast -p fret-ui-shadcn ...` used the wrong
  nextest profile flag and failed before running tests; a subsequent package-wide nextest attempt
  built too many unrelated shadcn integration tests and hit local rustc OOM. The final nextest
  commands above used `--cargo-profile dev-fast` plus `--lib`/test filtering and passed.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\radio-group\ui-gallery-radio-group-required-disabled-action-state.json --dir target\fret-diag-radio-group-required-disabled-action-state-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779358468383`.
- dedicated RadioGroup runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-radio-group-semantics\suite.json --dir target\fret-diag-radio-group-semantics-suite-required-disabled-v1 --session-auto --timeout-ms 600000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 2/2; `stage_counts={"passed":2}`; required/disabled row run id `1779358534244`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-radio-group-required-disabled-v1 --session-auto --timeout-ms 2400000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 26/26; `stage_counts={"passed":26}`; `reason_code_counts={}`;
    required/disabled row run id `1779360562701`.


## Checkbox Required Disabled Group Action-State Runtime Gate

- invariant:
  shadcn Checkbox required semantics live on each concrete checkbox control, not on the caller-owned
  fieldset/field-group shell. Disabled rows must suppress `focus`/`invoke` and label-forwarded
  toggles only for the disabled checkbox, while enabled sibling labels remain able to mutate their
  own checked state.
- finding:
  no Checkbox recipe/runtime defect was reproduced. The gate confirms that `Checkbox::required(true)`
  composes correctly with item-level `Checkbox::disabled(true)`, `Field::disabled(true)` chrome, and
  `FieldLabel::for_control(...)` forwarding in a grouped-control section.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/checkbox/required_disabled_group.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/checkbox/mod.rs`,
  `apps/fret-ui-gallery/src/ui/pages/checkbox.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-required-disabled-group-action-state.json`,
  `tools/diag-scripts/ui-gallery-checkbox-required-disabled-group-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-checkbox-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  upstream `repo-ref/ui/apps/v4/registry/new-york-v4/ui/checkbox.tsx` keeps disabled/required
  semantics on `CheckboxPrimitive.Root`; caller docs compose grouped rows through surrounding
  field/fieldset structure rather than widening Checkbox into a generic children container.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-checkbox-required-disabled-group-action-state-v1/sessions/1779363089819-213872/1779363101513/ai.packet`;
  focused runtime pack:
  `target/fret-diag-checkbox-required-disabled-group-action-state-v1/sessions/1779363089819-213872/share/1779363101513.zip`;
  dedicated suite summary:
  `target/fret-diag-checkbox-semantics-suite-required-disabled-group-v1/sessions/1779363128715-208764/suite.summary.json`;
  broad-suite summary:
  `target/fret-diag-shadcn-runtime-evidence-checkbox-required-disabled-group-v1/sessions/1779363773383-195668/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\checkbox\ui-gallery-checkbox-required-disabled-group-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-checkbox-required-disabled-group-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-checkbox-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\index.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\snippets\checkbox\required_disabled_group.rs apps\fret-ui-gallery\src\ui\snippets\checkbox\mod.rs apps\fret-ui-gallery\src\ui\pages\checkbox.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui-shadcn checkbox_required_exposes_required_semantics --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib checkbox_required_exposes_required_semantics`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app checkbox_ -- --nocapture`
  - result: passed; 7 tests.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app checkbox_required_disabled_group_snippet_keeps_required_and_disabled_action_state_on_concrete_controls`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_checkbox_required_disabled_group_action_state -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_checkbox_required_disabled_group_action_state`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\checkbox\ui-gallery-checkbox-required-disabled-group-action-state.json --dir target\fret-diag-checkbox-required-disabled-group-action-state-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779363101513`.
- dedicated Checkbox runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-checkbox-semantics\suite.json --dir target\fret-diag-checkbox-semantics-suite-required-disabled-group-v1 --session-auto --timeout-ms 600000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 3/3; `stage_counts={"passed":3}`; required/disabled group row run id
    `1779363388286`.
- broad runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite ui-gallery-shadcn-runtime-evidence --dir target\fret-diag-shadcn-runtime-evidence-checkbox-required-disabled-group-v1 --session-auto --timeout-ms 2400000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 27/27; `stage_counts={"passed":27}`; `reason_code_counts={}`;
    required/disabled group row run id `1779364468762`.

## ToggleGroup Disabled Item Action-State Runtime Gate

- invariant:
  single-mode shadcn/Radix ToggleGroup items behave as radio-button choices. Disabled item policy
  belongs to each concrete item: disabled items must expose `disabled=true`, suppress `focus` and
  `invoke`, and be skipped by roving focus without changing the selected value. The caller-owned
  field label/description shell must not become the action-state owner.
- finding:
  disabled item action suppression and roving focus behavior were correct, but the slice found a
  semantics completeness gap: single-mode ToggleGroup items exposed legacy `checked` only.
  `toggle_group_item_a11y_single(...)` now stamps explicit `checked_state=true|false` while
  preserving `checked`, so diagnostics and accessibility consumers can observe ToggleGroup radio
  checked state through the same structured channel as Checkbox and RadioGroup.
- implementation anchors:
  `ecosystem/fret-ui-kit/src/primitives/toggle_group.rs`,
  `ecosystem/fret-ui-shadcn/src/toggle_group.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/toggle_group/disabled_item_action_state.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/toggle_group/mod.rs`,
  `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-group-disabled-item-action-state.json`,
  `tools/diag-scripts/ui-gallery-toggle-group-disabled-item-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-toggle-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  upstream Radix ToggleGroup uses `role="radio"`/`aria-checked` for single mode and skips disabled
  items in roving focus. Fret maps that to concrete `radio_button` semantics with explicit
  `checked_state`, item-local disabled action suppression, and caller-owned label/field chrome.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-toggle-group-disabled-item-action-state-v2/sessions/1779371004637-185624/1779371026922/ai.packet`;
  focused runtime pack:
  `target/fret-diag-toggle-group-disabled-item-action-state-v2/sessions/1779371004637-185624/share/1779371026922.zip`;
  dedicated suite summary:
  `target/fret-diag-toggle-semantics-suite-disabled-item-v2/sessions/1779371056407-225584/suite.summary.json`;
  row-only suite summary:
  `target/fret-diag-toggle-group-disabled-item-suite-glob-v1/sessions/1779376207964-79492/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\toggle\ui-gallery-toggle-group-disabled-item-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-toggle-group-disabled-item-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-toggle-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check ecosystem\fret-ui-kit\src\primitives\toggle_group.rs ecosystem\fret-ui-shadcn\src\toggle_group.rs apps\fret-ui-gallery\src\ui\snippets\toggle_group\disabled_item_action_state.rs apps\fret-ui-gallery\src\ui\snippets\toggle_group\mod.rs apps\fret-ui-gallery\src\ui\pages\toggle_group.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui-kit toggle_group_item_a11y_single_uses_radio_role_and_checked --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit --lib toggle_group_item_a11y_single_uses_radio_role_and_checked`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-shadcn toggle_group_single_arrow_skips_disabled_and_exports_checked_state --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib toggle_group_single_arrow_skips_disabled_and_exports_checked_state`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app toggle_group_ -- --nocapture`
  - result: passed; 5 tests.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app toggle_group_disabled_item_action_state_snippet_keeps_roving_and_item_action_state_separate`
  - result: passed; 1 test.
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_toggle_group_disabled_item_action_state -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_toggle_group_disabled_item_action_state`
  - result: passed; 1 test.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\toggle\ui-gallery-toggle-group-disabled-item-action-state.json --dir target\fret-diag-toggle-group-disabled-item-action-state-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779371026922`.
- dedicated Toggle runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-toggle-semantics\suite.json --dir target\fret-diag-toggle-semantics-suite-disabled-item-v2 --session-auto --timeout-ms 600000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 2/2; `stage_counts={"passed":2}`; disabled-item row run id `1779371158503`.
- row-only diagnostics suite:
  `target\dev-fast\fretboard-dev.exe diag suite --glob 'tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-group-disabled-item-action-state.json' --dir target\fret-diag-toggle-group-disabled-item-suite-glob-v1 --session-auto --timeout-ms 600000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; disabled-item row run id `1779376227686`.
- broad-suite note:
  the script is promoted into `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`.
  A local full-suite attempt using `target\fret-diag-shadcn-runtime-evidence-toggle-group-disabled-item-v1`
  exceeded the outer shell timeout before reaching the new ToggleGroup row, so this slice uses the
  focused runtime, dedicated Toggle suite, and row-only suite as passing evidence.

## Menubar Disabled Item Action-State Runtime Gate

- invariant:
  shadcn/Radix Menubar disabled items must keep item-local ownership of disabled action-state:
  disabled items expose `disabled=true`, suppress `focus` and `invoke`, never dispatch their command,
  and are skipped by vertical roving focus. Enabled siblings and submenu triggers remain invokable,
  and the current programmatic roving focus target must export a semantics `focus` action even when
  it is outside default Tab traversal.
- finding:
  the gate found two real defects. First, Menubar content/submenu rows derived `focusable` by
  comparing the active roving item index to the flattened entry index, so separators and labels could
  shift the active tab stop onto the wrong row. Second, the `fret-ui` Pressable semantics bridge
  suppressed `actions.focus` whenever `PressableProps::focusable=false`, even if that Pressable was
  the current programmatic focus target used by roving focus. The fixes keep collection-index
  roving state aligned and preserve current-focus semantics without widening default focus
  traversal.
- implementation anchors:
  `crates/fret-ui/src/declarative/host_widget/semantics.rs`,
  `crates/fret-ui/src/declarative/tests/semantics.rs`,
  `ecosystem/fret-ui-shadcn/src/menubar.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/menubar/demo.rs`,
  `apps/fret-ui-gallery/src/spec.rs`,
  `apps/fret-ui-gallery/src/driver/runtime_driver.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-disabled-item-action-state.json`,
  `tools/diag-scripts/ui-gallery-menubar-disabled-item-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-menubar-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  Radix Menubar uses roving focus inside menu content, disabled items are skipped by focus movement
  and are not invokable, and submenu triggers remain enabled menu items. Fret maps this to concrete
  `menu_item` semantics, collection-position metadata, item-local disabled action suppression, and
  current-focus semantics for the active roving tab stop.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-menubar-disabled-item-action-state-v5/sessions/1779383380213-231888/1779383395682/ai.packet`;
  focused runtime pack:
  `target/fret-diag-menubar-disabled-item-action-state-v5/sessions/1779383380213-231888/share/1779383395682.zip`;
  dedicated suite summary:
  `target/fret-diag-menubar-semantics-suite-disabled-item-v1/sessions/1779383570688-235732/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools\diag-scripts\ui-gallery\menubar\ui-gallery-menubar-disabled-item-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\ui-gallery-menubar-disabled-item-action-state.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-menubar-semantics\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\suites\ui-gallery-shadcn-runtime-evidence\suite.json > $null`;
  `python -m json.tool tools\diag-scripts\index.json > $null`;
  `python tools\check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates\fret-ui\src\declarative\host_widget\semantics.rs crates\fret-ui\src\declarative\tests\semantics.rs apps\fret-ui-gallery\src\spec.rs apps\fret-ui-gallery\src\ui\snippets\menubar\demo.rs ecosystem\fret-ui-shadcn\src\menubar.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs crates\fret-diag-protocol\tests\script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo test --profile dev-fast -p fret-ui --lib declarative_pressable_current_focus_preserves_semantics_focus_action_outside_tab_order -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui --lib declarative_pressable_current_focus_preserves_semantics_focus_action_outside_tab_order`
  - result: passed; 1 test; run id `d8524063-5d0d-43d1-ad76-ae20ffd50f3a`.
  `cargo test --profile dev-fast -p fret-ui-shadcn menubar_disabled_item_skips_roving_focus_and_suppresses_action_state --lib -- --nocapture`
  - result: passed; 1 test.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib menubar_disabled_item_skips_roving_focus_and_suppresses_action_state`
  - result: passed; 1 test; run id `1b2c363f-cfb8-42fe-9db2-db5196ecd623`.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app menubar_demo_disabled_item_keeps_command_and_item_action_state_on_same_control`
  - result: passed; 1 test; run id `954f6256-29cd-4217-9c83-48301de43fbe`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_menubar_disabled_item_action_state`
  - result: passed; 1 test; run id `92c6d28f-ad5e-46c9-9cdf-b5841dc57b3e`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\menubar\ui-gallery-menubar-disabled-item-action-state.json --dir target\fret-diag-menubar-disabled-item-action-state-v5 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed; run id `1779383395682`.
- dedicated Menubar runtime suite:
  `target\dev-fast\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-menubar-semantics\suite.json --dir target\fret-diag-menubar-semantics-suite-disabled-item-v1 --session-auto --timeout-ms 600000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; disabled-item row run id `1779383578847`.
- broad-suite note:
  the script is promoted into `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`.
  This slice uses the focused runtime and dedicated Menubar suite as passing evidence to avoid
  burning the broad-suite timeout budget immediately after the previous long full-suite run.

## ContextMenu Disabled Item Action-State Runtime Gate

- invariant:
  shadcn/Radix ContextMenu disabled command items must keep concrete item ownership of disabled
  action-state: disabled rows expose `disabled=true`, suppress `focus` and `invoke`, never dispatch
  their command, and are skipped by roving focus. Enabled siblings remain focusable/invokable.
- finding:
  no ContextMenu recipe/runtime defect was reproduced for this invariant. The current recipe already
  suppresses disabled focus/invoke/command dispatch, preserves item collection metadata, and skips
  disabled rows during vertical roving focus. A separate diagnostics lifecycle concern remains: a
  default-lint dedicated suite can see duplicate `ui-gallery-context-menu-basic-content` nodes during
  transition captures; the row-only suite passes with `--no-suite-lint` and that duplicate-id lint is
  tracked as follow-up evidence rather than as an action-state failure.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/context_menu.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/basic.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-disabled-item-action-state.json`,
  `tools/diag-scripts/ui-gallery-context-menu-disabled-item-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-context-menu-semantics/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-context-menu/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  Radix ContextMenu uses roving focus inside menu content, disabled items are not invokable, and
  disabled items are skipped by focus movement. Fret maps this to concrete `menu_item` semantics,
  item-local disabled action suppression, command dispatch suppression, and collection-position
  metadata on the disabled row itself.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-context-menu-disabled-item-action-state-v2/sessions/1779387831262-20088/1779387844608/ai.packet`;
  focused runtime pack:
  `target/fret-diag-context-menu-disabled-item-action-state-v2/sessions/1779387831262-20088/share/1779387844608.zip`;
  passing row-only suite summary:
  `target/fret-diag-context-menu-semantics-suite-disabled-item-v3/sessions/1779388133420-237500/suite.summary.json`;
  default-lint follow-up artifact:
  `target/fret-diag-context-menu-semantics-suite-disabled-item-v2/sessions/1779387897940-40144/1779387958229-ui-gallery-context-menu-disabled-item-action-state/check.lint.json`.
- JSON/registry/formatting:
  `python -m json.tool tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-disabled-item-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery-context-menu-disabled-item-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-context-menu-semantics/suite.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-context-menu/suite.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json > $null`;
  `python -m json.tool tools/diag-scripts/index.json > $null`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check ecosystem/fret-ui-shadcn/src/context_menu.rs apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib context_menu_disabled_item_skips_roving_focus_and_suppresses_action_state`
  - result: passed; 1 test; run id `15532758-d144-470e-8280-0652eedd58ce`.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app context_menu_basic_disabled_item_keeps_command_and_item_action_state_on_same_control`
  - result: passed; 1 test; run id `5ac59f43-0c69-42d6-95de-726bd386d30b`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_context_menu_disabled_item_action_state`
  - result: passed; 1 test; run id `5ac78806-59db-45c3-84fd-146db8c77fa2`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-disabled-item-action-state.json --dir target/fret-diag-context-menu-disabled-item-action-state-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; run id `1779387844608`.
- dedicated ContextMenu row-only runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-context-menu-semantics/suite.json --dir target/fret-diag-context-menu-semantics-suite-disabled-item-v3 --session-auto --no-suite-lint --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`; disabled-item row run id `1779388147006`.
- default-lint suite concern:
  `target/dev-fast/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-context-menu-semantics/suite.json --dir target/fret-diag-context-menu-semantics-suite-disabled-item-v2 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: script row passed, then suite lint failed with `semantics.duplicate_test_id` for
    `ui-gallery-context-menu-basic-content`; follow-up artifact is
    `target/fret-diag-context-menu-semantics-suite-disabled-item-v2/sessions/1779387897940-40144/1779387958229-ui-gallery-context-menu-disabled-item-action-state/check.lint.json`.
- broad-suite note:
  the script is promoted into `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`.
  This slice uses the focused runtime plus row-only ContextMenu suite as passing evidence and keeps
  the duplicate-id transition lint as a narrow diagnostics follow-up.

## ContextMenu Basic Pointer-Open Keyboard Entry Runtime Gate

- invariant:
  shadcn/Radix ContextMenu pointer-open should initially focus the menu content panel; vertical
  keyboard navigation from that panel should enter the first enabled item, skip disabled items, keep
  command dispatch on enabled rows, and close on activation.
- finding:
  this follow-up found two real issues. The Basic example reused
  `ui-gallery-context-menu-basic-content` for both the DocSection wrapper and the overlay panel,
  causing default diagnostics lint to report duplicate ids. Separately, pointer-open ArrowDown
  reached the focused panel key handler but the handler saw empty first/last item targets because
  top-level panel entries bypassed the shared `ContextMenuContentRenderEnv` rendering path that
  records persistent focus targets. A focused `fret-ui` mechanism test confirmed key routing itself
  was correct for focused non-modal hit-test-inert overlays.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/context_menu.rs`,
  `crates/fret-ui/src/tree/tests/key_dispatch_barrier_root.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/basic.rs`,
  `apps/fret-ui-gallery/tests/popup_menu_narrow_surface.rs`,
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-keyboard-nav.json`,
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-right-click-last-action.json`,
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-touch-long-press-open.json`,
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-disabled-item-action-state.json`,
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-context-menu-basic-overlay-placement-trace.json`,
  `tools/diag-scripts/suites/ui-gallery-context-menu/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  Radix ContextMenu keeps content focus as the opened menu entry point and lets roving keyboard
  movement enter enabled items while skipping disabled rows. Fret maps that to a content-panel key
  handler backed by persistent first/last enabled item targets across overlay render reuse.
- evidence anchors:
  focused keyboard-nav AI packet:
  `target/fret-diag-context-menu-basic-keyboard-nav-focus-model-v3/sessions/1779402638361-240732/1779402658765/ai.packet`;
  focused keyboard-nav pack:
  `target/fret-diag-context-menu-basic-keyboard-nav-focus-model-v3/sessions/1779402638361-240732/share/1779402658765.zip`;
  default-lint ContextMenu semantics suite summary:
  `target/fret-diag-context-menu-semantics-suite-panel-id-v1/sessions/1779403614963-229576/suite.summary.json`;
  promoted ContextMenu suite summary:
  `target/fret-diag-context-menu-suite-keyboard-entry-v2/sessions/1779404255883-237716/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-keyboard-nav.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-right-click-last-action.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-touch-long-press-open.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-disabled-item-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-context-menu-basic-overlay-placement-trace.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-context-menu/suite.json > $null`;
  `python -m json.tool tools/diag-scripts/index.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check ecosystem/fret-ui-shadcn/src/context_menu.rs crates/fret-ui/src/tree/tests/key_dispatch_barrier_root.rs apps/fret-ui-gallery/src/ui/snippets/context_menu/basic.rs apps/fret-ui-gallery/tests/popup_menu_narrow_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib context_menu_pointer_open_arrow_down_enters_first_enabled_item context_menu_pointer_open_arrow_down_enters_first_enabled_checkbox_or_radio_item context_menu_disabled_item_skips_roving_focus_and_suppresses_action_state`
  - result: passed; 3 tests; run id `5960047a-484b-4c5d-8a9e-1f7c86ed1dce`.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui --lib key_events_route_to_focused_non_modal_occluding_overlay_layer`
  - result: passed; 1 test; run id `1f37ca4f-b615-44f7-a238-c4431a6cca3d`.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --test popup_menu_narrow_surface context_menu_basic_snippet_uses_a_unique_overlay_panel_test_id context_menu_demo_snippet_uses_a_unique_overlay_panel_test_id`
  - result: passed; 2 tests; run id `45c8dc6b-13b8-4b9f-b3d9-f6516e4411e7`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_context_menu_basic_keyboard_nav script_v2_roundtrip_ui_gallery_context_menu_disabled_item_action_state script_v2_roundtrip_ui_gallery_context_menu_basic_right_click_last_action script_v2_roundtrip_ui_gallery_context_menu_basic_touch_long_press_open script_v2_roundtrip_ui_gallery_context_menu_basic_overlay_placement_trace`
  - result: passed; 5 tests; run id `c771179b-7984-479c-968a-7b836d24985f`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-keyboard-nav.json --dir target/fret-diag-context-menu-basic-keyboard-nav-focus-model-v3 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; run id `1779402658765`.
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-disabled-item-action-state.json --dir target/fret-diag-context-menu-disabled-item-action-state-panel-id-v2 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; run id `1779402722466`.
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-right-click-last-action.json --dir target/fret-diag-context-menu-basic-right-click-last-action-panel-id-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; run id `1779402810724`.
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-touch-long-press-open.json --dir target/fret-diag-context-menu-basic-touch-long-press-open-panel-id-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; run id `1779403249948`.
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-context-menu-basic-overlay-placement-trace.json --dir target/fret-diag-context-menu-basic-overlay-placement-trace-panel-id-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 300000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; run id `1779403333605`.
- runtime suites:
  `target/dev-fast/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-context-menu-semantics/suite.json --dir target/fret-diag-context-menu-semantics-suite-panel-id-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed 1/1; `stage_counts={"passed":1}`.
  `target/dev-fast/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-context-menu/suite.json --dir target/fret-diag-context-menu-suite-keyboard-entry-v2 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed 4/4; `stage_counts={"passed":4}`; keyboard-nav row run id `1779404347719`.

## ContextMenu Basic Typeahead Runtime Gate

- invariant:
  shadcn/Radix ContextMenu typeahead should run through the same menu roving path as keyboard
  navigation: after pointer-open focuses the content panel and ArrowDown enters the first enabled
  item, a printable matching key should move focus to the matching menu item, keep the overlay open
  until activation, and activation should dispatch the enabled row command. No-match input should
  preserve current focus, and stale prefix input should clear after the configured timeout.
- finding:
  no ContextMenu recipe/runtime defect was reproduced. The new synthetic fixture proves
  ContextMenu typeahead, no-match preservation, and buffer-timeout reset. The new UI Gallery runtime
  gate proves the Basic page's right-click path enters `Back`, `r` focuses `Reload`, Enter dispatches
  `ui_gallery.context_menu.basic.reload`, and the menu closes.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/tests/fixtures/recipe_typeahead_cases_v1.json`,
  `ecosystem/fret-ui-shadcn/tests/recipe_typeahead_mechanism_harness.rs`,
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-typeahead-reload.json`,
  `tools/diag-scripts/suites/ui-gallery-context-menu/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  Radix menu typeahead searches enabled items in the active menu content without dismissing the
  overlay. Fret maps that to `fret-ui-kit` menu roving prefix typeahead and a ContextMenu
  recipe-owned content panel entry path.
- evidence anchors:
  focused typeahead AI packet:
  `target/fret-diag-context-menu-basic-typeahead-reload-v1/sessions/1779472134533-73472/1779472143821/ai.packet`;
  focused typeahead pack:
  `target/fret-diag-context-menu-basic-typeahead-reload-v1/sessions/1779472134533-73472/share/1779472143821.zip`;
  refreshed ContextMenu suite summary:
  `target/fret-diag-context-menu-suite-typeahead-reload-v1/sessions/1779472199452-77576/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool ecosystem/fret-ui-shadcn/tests/fixtures/recipe_typeahead_cases_v1.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-typeahead-reload.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-context-menu/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/test_check_diag_scripts_registry.py`;
  `cargo fmt -p fret-ui-shadcn -p fret-diag-protocol`
  - result: passed.
- focused Rust/protocol gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --test recipe_typeahead_mechanism_harness mechanism_harness_recipe_typeahead_cases_match_oracles --no-fail-fast --no-capture`
  - result: passed; run id `91e6eccf-10ba-4899-8ef5-44add5f22ff7`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_context_menu_basic_typeahead_reload --no-fail-fast --no-capture`
  - result: passed; run id `46268cd8-c84a-4cdd-a195-6b6517b8caa3`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-typeahead-reload.json --dir target/fret-diag-context-menu-basic-typeahead-reload-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; run id `1779472143821`.
- runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-context-menu --dir target/fret-diag-context-menu-suite-typeahead-reload-v1 --session-auto --include-triage --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed 5/5; typeahead row run id `1779472350644`.

## ContextMenu Submenu Typeahead Runtime Gate

- invariant:
  nested ContextMenu content should use the same Radix/menu roving typeahead semantics as root menu
  content after keyboard-open focus transfer: once `ArrowRight` opens the submenu and focus moves
  to the first submenu item, a printable matching key should search that submenu content, keep the
  menu path open until activation, and activation should dispatch the focused submenu action.
- finding:
  no ContextMenu recipe/runtime defect was reproduced. The synthetic fixture proves submenu-open
  typeahead moves focus from `Save Page...` to `Name Window...` while keeping the ContextMenu open.
  The new UI Gallery runtime gate proves the Submenu page's right-click path enters root roving
  focus, opens `More Tools`, typeahead-focuses `Name Window...`, dispatches
  `ui_gallery.context_menu.submenu.name_window`, and closes the menu.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/tests/fixtures/recipe_typeahead_cases_v1.json`,
  `ecosystem/fret-ui-shadcn/tests/recipe_typeahead_mechanism_harness.rs`,
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-typeahead-name-window.json`,
  `tools/diag-scripts/suites/ui-gallery-context-menu/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- source-alignment anchor:
  Radix menu submenu content participates in the same menu roving/typeahead model after focus is
  transferred into the submenu. Fret maps that to `fret-ui-kit::menu::sub_content` roving prefix
  typeahead and ContextMenu recipe-owned submenu panel rendering.
- evidence anchors:
  focused typeahead AI packet:
  `target/fret-diag-context-menu-submenu-typeahead-name-window-v1/sessions/1779473854793-15920/1779473864097/ai.packet`;
  focused typeahead pack:
  `target/fret-diag-context-menu-submenu-typeahead-name-window-v1/sessions/1779473854793-15920/share/1779473864097.zip`;
  refreshed ContextMenu suite summary:
  `target/fret-diag-context-menu-suite-submenu-typeahead-v1/sessions/1779473909388-74436/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool ecosystem/fret-ui-shadcn/tests/fixtures/recipe_typeahead_cases_v1.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-typeahead-name-window.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-context-menu/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt -p fret-ui-shadcn -p fret-diag-protocol --check`
  - result: passed.
- focused Rust/protocol gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --test recipe_typeahead_mechanism_harness mechanism_harness_recipe_typeahead_cases_match_oracles --no-fail-fast --no-capture`
  - result: passed; run id `2137d669-1531-477e-a957-7dcee779c2ff`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_context_menu_submenu_typeahead_name_window --no-fail-fast --no-capture`
  - result: passed; run id `0d475764-1f1d-4c86-b08d-c2e514192df2`.
- build:
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-submenu-typeahead-name-window.json --dir target/fret-diag-context-menu-submenu-typeahead-name-window-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed; run id `1779473864097`.
- runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-context-menu --dir target/fret-diag-context-menu-suite-submenu-typeahead-v1 --session-auto --include-triage --timeout-ms 1000000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  - result: passed 6/6; submenu typeahead row run id `1779474106732`;
    `focus_mismatch_total=0`.

## HitTestOnly Stale Path-Cache Runtime Gate

- invariant:
  a move-only hit-test path-cache entry must be rejected when a higher-z sibling moves under the
  pointer across a cached/root boundary; fallback hit testing must route to the higher-z sibling
  and refresh the path cache for subsequent moves.
- finding:
  no new `fret-ui` mechanism defect was reproduced. The gap was runtime evidence: synthetic tests
  already covered the stale lower-child path and dispatch-level pointer moves, but UI Gallery did
  not have a cached-root surface that produced a stale path-cache miss plus a refreshed hit in a
  deterministic diagnostics run.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/harness/hit_test_only_paint_cache_probe.rs`,
  `tools/diag-scripts/ui-gallery/diag/ui-gallery-hit-test-only-stale-path-cover-move.json`,
  `tools/diag-scripts/suites/ui-gallery-hit-test-only-paint-cache/suite.json`,
  `tools/diag-scripts/index.json`, and
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-hit-test-only-stale-path-cover-move-v1/sessions/1779415325602-253012/1779415611375/ai.packet`;
  focused runtime pack:
  `target/fret-diag-hit-test-only-stale-path-cover-move-v1/sessions/1779415325602-253012/share/1779415611375.zip`;
  dedicated suite summary:
  `target/fret-diag-hit-test-only-paint-cache-suite-stale-path-v1/sessions/1779415674198-166024/suite.summary.json`.
- JSON/registry/formatting:
  `python -m json.tool tools/diag-scripts/ui-gallery/diag/ui-gallery-hit-test-only-stale-path-cover-move.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-hit-test-only-paint-cache/suite.json > $null`;
  `python -m json.tool tools/diag-scripts/index.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/previews/pages/harness/hit_test_only_paint_cache_probe.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `git diff --check`
  - result: passed.
- focused Rust gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_hit_test_only_paint_cache_probe_sweep script_v2_roundtrip_ui_gallery_hit_test_only_stale_path_cover_move`
  - result: passed; 2 tests; run id `967b9ba4-5eb8-4a7c-847e-9310162471fa`.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer pointer_move_dispatch_rejects_stale_path_when_higher_z_sibling_moves_under_pointer prepaint_interaction_cache_root_move_invalidates_stale_root_only_hit_path`
  - result: passed; 3 tests; run id `3214e28c-f4c6-45a9-b838-ae016e824ede`.
- build/check:
  `cargo check --profile dev-fast -p fret-ui-gallery --features gallery-dev`
  - result: passed.
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/diag/ui-gallery-hit-test-only-stale-path-cover-move.json --dir target/fret-diag-hit-test-only-stale-path-cover-move-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779415611375`.
- dedicated runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-hit-test-only-paint-cache --dir target/fret-diag-hit-test-only-paint-cache-suite-stale-path-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 2/2; `stage_counts={"passed":2}`; stale-path row run id `1779416013201`.

## State-sensitive CachedSubtree Cache-Key Authoring Guard

- invariant:
  cached subtree output that depends on caller state while the callsite identity stays stable must
  encode that state in the explicit cache key. Boolean state should use a typed helper instead of
  ad-hoc integer coercion so stale retained/replayed metadata risks are visible in review.
- finding:
  no new mechanism or recipe defect was reproduced. The runtime disabled/action-state path was
  already covered by the moving cached Combobox gate; this slice closes the authoring/API gap that
  made the boolean dependency less reviewable.
- implementation anchors:
  `crates/fret-ui/src/cache_key.rs`,
  `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/resizable/moving_cached_combobox.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/resizable/notes.rs`,
  `apps/fret-ui-gallery/tests/resizable_docs_surface.rs`, and
  `docs/component-author-guide.md`.
- focused Rust gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui bool_key_tracks_boolean_state --no-fail-fast --no-capture`
  - result: passed; run id `d1cd6ef8-cf4a-4b66-8123-fb469db4ceaf`.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit cached_subtree_props_bool_key_tracks_state_sensitive_cached_content --no-fail-fast --no-capture`
  - result: passed; run id `1b49d112-fa2c-41cb-868a-51984555a007`.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --features gallery-dev resizable_snippets_stay_copyable_and_docs_aligned --no-fail-fast --no-capture`
  - result: passed; run id `e9808d7f-2af3-41a3-bf4d-ceeb03b858bc`.
- build:
  `cargo build --profile dev-fast -p fret-ui-gallery --features gallery-dev`
  - result: passed.

## Retained Table Row-Pinning Selected/Action-State Gate

- invariant:
  a pinned retained Table row that survives a pagination/window boundary must keep fresh semantics
  metadata. Presence alone is insufficient; the row must still expose `selected=true` after pointer
  selection and `invoke=true` after it is pinned and page 2 is shown.
- finding:
  no retained Table stale selected/invoke defect was reproduced. The slice closes the diagnostics
  gap where the existing keep-pinned row gate asserted row existence but did not verify retained
  row semantics/action-state freshness after pagination.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/table/ui-gallery-table-retained-row-pinning-keep-pinned-true.json`
  now starts directly on the retained Table Torture page, selects row 0, verifies selected/invoke
  semantics, pins row 0, advances to page 2, and verifies row 0 still exists with fresh
  selected/invoke semantics.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/table/ui-gallery-table-retained-row-pinning-keep-pinned-true.json`,
  `tools/diag-scripts/suites/ui-gallery-table-retained/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`, and
  `apps/fret-ui-gallery/src/ui/previews/gallery/torture/table_retained_torture.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-table-retained-row-pinning-selected-action-v1/sessions/1779480713460-19924/1779480725739/ai.packet`;
  focused runtime pack:
  `target/fret-diag-table-retained-row-pinning-selected-action-v1/sessions/1779480713460-19924/share/1779480725739.zip`;
  full retained Table suite summary:
  `target/fret-diag-table-retained-suite-row-pinning-selected-action-v1/sessions/1779480750237-48512/suite.summary.json`.
- JSON/registry:
  `python -m json.tool tools/diag-scripts/ui-gallery/table/ui-gallery-table-retained-row-pinning-keep-pinned-true.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_table_retained_row_pinning_keep_pinned_true --no-fail-fast --no-capture`
  - result: passed; run id `8dbf737e-559d-4dfd-b340-15def5ed7771`.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/table/ui-gallery-table-retained-row-pinning-keep-pinned-true.json --dir target/fret-diag-table-retained-row-pinning-selected-action-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779480725739`.
- retained Table runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-table-retained --dir target/fret-diag-table-retained-suite-row-pinning-selected-action-v1 --session-auto --include-triage --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 7/7; strengthened keep-pinned row run id `1779480853485`.
- bounded evidence:
  `target/dev-fast/fretboard-dev.exe diag query test-id target/fret-diag-table-retained-row-pinning-selected-action-v1/sessions/1779480713460-19924 ui-gallery-table-retained-row-0 --json --top 10`
  found row 0 and its visible cells once in the final focused bundle.
  `target/dev-fast/fretboard-dev.exe diag slice target/fret-diag-table-retained-row-pinning-selected-action-v1/sessions/1779480713460-19924 --test-id ui-gallery-table-retained-row-0 --json --max-matches 2 --max-ancestors 6`
  shows row 0 under `ui-gallery-table-retained-torture-root` with role `list_item`,
  `flags.selected=true`, and `actions.invoke=true`.

## FileTree Torture Retained Hierarchy/Action-State Gate

- invariant:
  a retained FileTree row hierarchy must keep fresh semantics metadata across collapse/expand
  detach/reattach and retained virtual-list scroll escape. Presence alone is insufficient: root,
  folder, and leaf rows must expose correct hierarchy levels, expanded state where applicable,
  selected-state transfer, and invoke action availability after the selected leaf detaches and
  reattaches.
- finding:
  no FileTree retained hierarchy/action-state defect was reproduced. The first focused draft
  exposed diagnostics authoring drift where an immediate assertion could race semantics predicate
  convergence; the failure bundle already contained the expected `level=2` folder semantics. The
  final gate uses bounded `wait_until` predicates for semantics convergence while retaining strict
  outcome checks.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/file-tree/ui-gallery-file-tree-torture-retained-hierarchy-action-state.json`
  starts directly on File Tree Torture with `FRET_UI_GALLERY_FILE_TREE_ROOTS=20`, verifies root,
  folder, and leaf semantics, collapses/re-expands folder `1000000`, selects leaf `2000000`,
  scrolls it out through a retained virtual-list escape reconcile, then scrolls back and proves the
  leaf reattaches with `level=3`, `selected=true`, and `invoke=true`.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/file-tree/ui-gallery-file-tree-torture-retained-hierarchy-action-state.json`,
  `tools/diag-scripts/ui-gallery-file-tree-torture-retained-hierarchy-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-file-tree-retained/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`, and
  `apps/fret-ui-gallery/src/ui/previews/gallery/torture/file_tree_torture.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-file-tree-retained-hierarchy-action-state-v4/sessions/1779499403324-13196/1779499421903/ai.packet`;
  focused runtime pack:
  `target/fret-diag-file-tree-retained-hierarchy-action-state-v4/sessions/1779499403324-13196/share/1779499421903.zip`;
  dedicated suite summary:
  `target/fret-diag-file-tree-retained-suite-hierarchy-action-state-v1/sessions/1779499651434-70912/suite.summary.json`.
- JSON/registry:
  `python -m json.tool tools/diag-scripts/ui-gallery/file-tree/ui-gallery-file-tree-torture-retained-hierarchy-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery-file-tree-torture-retained-hierarchy-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-file-tree-retained/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_file_tree_torture_retained_hierarchy_action_state --no-fail-fast --no-capture`
  - result: passed; run id `1532f582-9abf-48b9-bd1e-267fa00a6f3b`.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/file-tree/ui-gallery-file-tree-torture-retained-hierarchy-action-state.json --dir target/fret-diag-file-tree-retained-hierarchy-action-state-v4 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed; run id `1779499421903`.
- retained FileTree runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-file-tree-retained --dir target/fret-diag-file-tree-retained-suite-hierarchy-action-state-v1 --session-auto --include-triage --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  - result: passed 1/1; run id `1779499662204`; summary:
    `target/fret-diag-file-tree-retained-suite-hierarchy-action-state-v1/sessions/1779499651434-70912/suite.summary.json`.
- bounded evidence:
  `target/dev-fast/fretboard-dev.exe diag slice target/fret-diag-file-tree-retained-hierarchy-action-state-v4/sessions/1779499403324-13196 --test-id ui-gallery-file-tree-node-2000000 --json --max-matches 2 --max-ancestors 8`
  shows the reattached leaf under `ui-gallery-file-tree-root` with role `tree_item`, `level=3`,
  `flags.selected=true`, and `actions.invoke=true`.

## Inspector Torture Row-Root Selected/Action-State Gate

- invariant:
  a retained Inspector row root must keep fresh collection semantics and action state across
  detach/reattach and retained virtual-list scroll escape. Presence alone is insufficient: the row
  root must still expose `ListItem` semantics, collection metadata, `selected`, and `invoke` after
  the selected row leaves and reenters the retained window.
- finding:
  no Inspector retained row-root selected/action-state defect was reproduced. The gate proved the
  row root can be selected, detached, and reattached with fresh semantics while the retained
  virtual list still reports detach/reattach reconciliation.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-row-root-selected-action-state-bounce.json`
  starts directly on the dev-only Inspector Torture page, stamps retained row-root `ListItem`
  semantics with collection metadata plus `selected`/`invoke`, selects row 2, scrolls it out of the
  retained window, and proves the row reattaches with `selected=true` and `invoke=true`.
- implementation anchors:
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-row-root-selected-action-state-bounce.json`,
  `tools/diag-scripts/ui-gallery-inspector-torture-row-root-selected-action-state-bounce.json`,
  `tools/diag-scripts/suites/ui-gallery-inspector-torture-row-root-selected-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`, and
  `apps/fret-ui-gallery/src/ui/previews/gallery/torture/inspector_torture.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-inspector-row-root-selected-action-state-v3/sessions/1779506053131-100636/1779506078170/ai.packet`;
  focused runtime pack:
  `target/fret-diag-inspector-row-root-selected-action-state-v3/sessions/1779506053131-100636/share/1779506078170.zip`;
  dedicated suite summary:
  `target/fret-diag-inspector-row-root-selected-action-state-suite-v1/sessions/1779506349279-97672/suite.summary.json`.
- JSON/registry:
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt -p fret-diag-protocol --check`;
  `cargo fmt -p fret-ui-gallery --check`.
  - result: passed.
- protocol roundtrip:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_inspector_torture_row_root_selected_action_state_bounce --no-fail-fast --no-capture`
  - result: passed; run id `2f89ec06-073d-4d7a-ac52-ab73b0abc156`.
- gallery authoring guards:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_inspector_torture_uses_fixed_row_text_roles gallery_inspector_torture_stamps_row_root_semantics_and_action_state --no-fail-fast --no-capture`
  - result: passed; run id `3a4ec18b-57b6-4348-98ea-ebcbacde9fa9`.
- focused runtime diagnostics:
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery-inspector-torture-row-root-selected-action-state-bounce.json --dir target/fret-diag-inspector-row-root-selected-action-state-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed; run id `1779506078170`.
- dedicated runtime suite:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-inspector-torture-row-root-selected-action-state --dir target/fret-diag-inspector-row-root-selected-action-state-suite-v1 --session-auto --include-triage --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 1/1; summary
    `target/fret-diag-inspector-row-root-selected-action-state-suite-v1/sessions/1779506349279-97672/suite.summary.json`.

## Fresh Continuation Verification 2026-05-23

- scope:
  after recovering session `019e5316-ad17-74d0-a3ee-97a095ee099a`, revalidated the dirty worktree
  slice set covering UI Kit List row-root semantics, Windowed Rows surface refresh policy,
  FileTree retained hierarchy/action-state, Inspector retained row-root selected/action-state, and
  Card retained-analysis precondition scripts. The first Inspector suite attempt used a prebuilt
  binary and failed only the diagnostics feature preflight (`tooling.launch.failed`); it was
  discarded as non-evidence and rerun with an inspectable `cargo run` launch.
- formatting/build:
  `cargo fmt --check --package fret-ui-kit --package fret-ui-gallery --package fret-diag --package fret-diag-protocol`
  - result: passed.
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`
  - result: passed.
- focused Rust gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit list_virtualized_copyable_retained_debug_row_ids_target_row_semantics --no-fail-fast --no-capture`
  - result: passed; run id `f3b180b4-4104-407e-8e5e-46e440dd3317`.
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery gallery_inspector_torture_stamps_row_root_semantics_and_action_state --no-fail-fast --no-capture`
  - result: passed; run id `13258b92-ce0b-4668-972e-f483fc8ba2ed`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag windowed_rows_gallery_surface_scroll_refresh_uses_offset_and_repaint_gates --no-fail-fast --no-capture`
  - result: passed; run id `35e3dd47-2c95-4d73-bb3f-bb4fe76450eb`.
- protocol roundtrip gates:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_windowed_rows_surface_scroll_refresh --no-fail-fast --no-capture`
  - result: passed; run id `ba0b3d36-536b-4099-8edf-4cb83b73663e`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_ui_kit_list_window_boundary_scroll --no-fail-fast --no-capture`
  - result: passed; run id `5ed3bee1-776b-4bc2-aa10-3a2eb3f48b59`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_file_tree_torture_retained_hierarchy_action_state --no-fail-fast --no-capture`
  - result: passed; run id `a2ec4ecb-e866-4278-957e-0700e8d9d6b8`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_inspector_torture_row_root_selected_action_state_bounce --no-fail-fast --no-capture`
  - result: passed; run id `b418f5d1-cb7b-4261-8630-4760a4eff71d`.
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_card_memory_retained_analysis_scripts --no-fail-fast --no-capture`
  - result: passed; run id `79d25d77-bdb7-4d85-88ce-8af2a0d8592f`.
- fresh runtime suites:
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-inspector-torture-row-root-selected-action-state --dir target/fret-diag-inspector-row-root-selected-action-state-fresh-v2 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 1/1; run id `1779515633307`; summary
    `target/fret-diag-inspector-row-root-selected-action-state-fresh-v2/sessions/1779515556317-60168/suite.summary.json`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-ui-kit-list-retained --dir target/fret-diag-ui-kit-list-row-root-semantics-fresh-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 1/1; run id `1779516015401`; summary
    `target/fret-diag-ui-kit-list-row-root-semantics-fresh-v1/sessions/1779516004847-57072/suite.summary.json`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-file-tree-retained --dir target/fret-diag-file-tree-retained-hierarchy-action-state-fresh-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 1/1; run id `1779516059547`; summary
    `target/fret-diag-file-tree-retained-hierarchy-action-state-fresh-v1/sessions/1779516044541-31432/suite.summary.json`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-windowed-rows-surface --dir target/fret-diag-windowed-rows-surface-scroll-refresh-fresh-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 1/1; run id `1779516287552`; summary
    `target/fret-diag-windowed-rows-surface-scroll-refresh-fresh-v1/sessions/1779516276970-56592/suite.summary.json`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-card-retained-analysis-navnone --dir target/fret-diag-card-retained-analysis-navnone-fresh-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 4/4; row run ids `1779516346989`, `1779516515594`,
    `1779516542022`, and `1779516664699`; summary
    `target/fret-diag-card-retained-analysis-navnone-fresh-v1/sessions/1779516342911-64120/suite.summary.json`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-card-preview-retained-bisect-navnone --dir target/fret-diag-card-preview-retained-bisect-navnone-fresh-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 3/3; row run ids `1779516887535`, `1779517007443`, and
    `1779517083664`; summary
    `target/fret-diag-card-preview-retained-bisect-navnone-fresh-v1/sessions/1779516867410-58496/suite.summary.json`.
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-card-preview-retained-hotspots-navnone --dir target/fret-diag-card-preview-retained-hotspots-navnone-fresh-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  - result: passed 4/4; row run ids `1779517181481`, `1779517302911`,
    `1779517416967`, and `1779517508009`; summary
    `target/fret-diag-card-preview-retained-hotspots-navnone-fresh-v1/sessions/1779517160250-1544/suite.summary.json`.

## View Cache Cached Popover Relation/Action-State Gate

- invariant:
  a cached View Cache Popover must keep fresh trigger `invoke` action state and a valid
  `controls` relation to the dialog wrapper across close, cached counter mutation, and reopen.
  Presence alone is not enough: the trigger must clear the relation when closed and reestablish it
  after the cached subtree is reused.
- finding:
  no View Cache Popover relation/action-state defect was reproduced. The trigger kept
  `invoke=true`, the `controls` edge resolved to the dedicated dialog wrapper test id while open,
  the edge cleared after close, and the same relation/action-state contract returned after a
  cached counter mutation and reopen.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-cached-popover-relation-action-state.json`
  starts directly on the View Cache harness page, stamps
  `ui-gallery-view-cache-popover-dialog` on the Popover dialog wrapper, asserts the
  `ui-gallery-view-cache-popover-content` panel, and drives close/reopen across a cached counter
  mutation while checking `expanded`, `invoke`, and `controls`.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/harness/view_cache.rs`,
  `ecosystem/fret-ui-shadcn/src/popover.rs`,
  `tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-cached-popover-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-view-cache/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-view-cache-cached-popover-relation-action-state-v1/sessions/1779519713733-54180/1779519791015/ai.packet`;
  focused runtime pack:
  `target/fret-diag-view-cache-cached-popover-relation-action-state-v1/sessions/1779519713733-54180/share/1779519791015.zip`;
  dedicated suite summary:
  `target/fret-diag-view-cache-suite-cached-popover-relation-v1/sessions/1779519979352-64968/suite.summary.json`.
- run results:
  `python tools/check_diag_scripts_registry.py`;
  `python -m json.tool tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-cached-popover-relation-action-state.json > $null`;
  `cargo fmt --package fret-ui-shadcn --package fret-ui-gallery --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_view_cache_cached_popover_relation_action_state --no-fail-fast --no-capture`
  (run id `d2dea496-7781-4279-8e64-185b3271844c`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib popover_trigger_exposes_expanded_and_controls_semantics --no-fail-fast --no-capture`
  (run id `0b36665f-a777-46bd-a81a-f107e9c0fdc0`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/view-cache/ui-gallery-view-cache-cached-popover-relation-action-state.json --dir target/fret-diag-view-cache-cached-popover-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (run id `1779519791015`);
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-view-cache --dir target/fret-diag-view-cache-suite-cached-popover-relation-v1 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (suite run id `1779519979352-64968`, 3/3 passed).

## ScrollArea Arm Content Growth Click Stability

- invariant:
  the ScrollArea Arm content growth path must prove target visibility, action exposure, command
  dispatch, and growth publication separately; a transient `Armed` badge is not the only valid
  oracle.
- finding:
  no mechanism defect was reproduced. The old promoted `content-growth` gate was racing the
  intermediate `Armed` badge even though the click already reached the target and published
  `ui_gallery.scroll_area.drag_baseline.arm_growth`. The focused gate now proves the action-state
  split directly, and the full ScrollArea suite passes after hardening the old drag scripts.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-arm-content-growth-click-stability.json`
  starts directly on the ScrollArea diagnostics page, proves the reset button, checks the arm
  button bounds and `invoke` semantics, confirms `/shell/last_action`, and then exercises two
  arm/reset cycles before the growth/drag checks.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/diagnostics/scroll_area/drag_baseline.rs`,
  `apps/fret-ui-gallery/src/ui/pages/scroll_area.rs`,
  `apps/fret-ui-gallery/src/ui/content.rs`,
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-arm-content-growth-click-stability.json`,
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json`,
  `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-pointer-cancel-release.json`,
  `tools/diag-scripts/suites/ui-gallery-scroll-area/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-scrollbar-arm-content-growth-click-stability-v1/sessions/1779521866482-61600/1779521876031/ai.packet`;
  focused runtime pack:
  `target/fret-diag-scrollbar-arm-content-growth-click-stability-v1/sessions/1779521866482-61600/share/1779521876031.zip`;
  old-suite failure bundle:
  `target/fret-diag-scroll-area-suite-arm-click-stability-v1/sessions/1779521906514-77508/1779522373786-script-step-0016-wait_until-timeout`;
  dedicated suite summary:
  `target/fret-diag-scroll-area-suite-arm-click-stability-v2/sessions/1779523448787-60024/suite.summary.json`.
- run results:
  `python tools/check_diag_scripts_registry.py`;
  `python -m json.tool tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-arm-content-growth-click-stability.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-pointer-cancel-release.json > $null`;
  `cargo fmt --package fret-ui-gallery --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_scrollbar --no-fail-fast --no-capture`
  (run id `c63b5638-b317-4a31-a09b-7a0efb5923b5`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-arm-content-growth-click-stability.json --dir target/fret-diag-scrollbar-arm-content-growth-click-stability-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779521876031`);
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json --dir target/fret-diag-scrollbar-drag-baseline-content-growth-arm-hardening-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779523331279`);
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-pointer-cancel-release.json --dir target/fret-diag-scrollbar-drag-pointer-cancel-release-arm-hardening-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779523399032`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-scroll-area --dir target/fret-diag-scroll-area-suite-arm-click-stability-v2 --session-auto --timeout-ms 900000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (suite run id `1779523448787-60024`, 8/8 passed).

## Chrome Torture Cached Overlay Action Dispatch

- invariant:
  overlay controls rendered inside the Chrome Torture cached overlay body must keep fresh action
  exposure, command dispatch, menu close, and outside-press dismissal state. A focused gate should
  prove routing and state publication without relying on an overlay-page-specific focus target.
- finding:
  no mechanism defect was reproduced. Reset, DropdownMenu Apple, ContextMenu Action, and Popover
  outside-press dismissal all dispatched through `/shell/last_action`; DropdownMenu and ContextMenu
  content closed after activation; and Popover dismissal published both `ui-gallery-popover-dismissed`
  and `ui-gallery-overlay-underlay-activated`. The first focused draft failed only on an
  over-specific `focus_is(ui-gallery-overlay-underlay)` oracle after dismissal.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-overlay-action-dispatch.json`
  starts directly on the dev-only Chrome Torture page, uses `ui-gallery-content-scroll` to keep the
  cached overlay controls visible, checks `invoke` action availability, and captures a bounded
  bundle after the dispatch/dismissal sequence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/chrome_torture.rs`,
  `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/overlay.rs`,
  `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/overlay/widgets.rs`,
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-overlay-action-dispatch.json`,
  `tools/diag-scripts/suites/ui-gallery-chrome-torture-overlay-action-dispatch/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-chrome-torture-overlay-action-dispatch-v2/sessions/1779526726749-55768/1779526747845/ai.packet`;
  focused runtime pack:
  `target/fret-diag-chrome-torture-overlay-action-dispatch-v2/sessions/1779526726749-55768/share/1779526747845.zip`;
  first-draft focus-oracle failure packet:
  `target/fret-diag-chrome-torture-overlay-action-dispatch-v1/sessions/1779526230419-78920/1779526251707/ai.packet`;
  dedicated suite summary:
  `target/fret-diag-chrome-torture-overlay-action-dispatch-suite-v1/sessions/1779526947605-48036/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-overlay-action-dispatch.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-chrome-torture-overlay-action-dispatch/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_chrome_torture_overlay_action_dispatch --no-fail-fast --no-capture`
  (run id `d7d3aed1-c539-4a17-a6f6-656e7fa662e9`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/perf/ui-gallery-chrome-torture-overlay-action-dispatch.json --dir target/fret-diag-chrome-torture-overlay-action-dispatch-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779526747845`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-chrome-torture-overlay-action-dispatch --dir target/fret-diag-chrome-torture-overlay-action-dispatch-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779526968150`, suite run id `1779526947605-48036`, 1/1 passed).

## Material3 State Matrix Chip Action State

- invariant:
  cacheable Material 3 State Matrix chip controls must expose explicit checked-state semantics,
  disabled action suppression, primary/trailing action availability, and action dispatch through
  the same runtime path used by the UI Gallery shell.
- finding:
  a real Material3 recipe gap was reproduced and fixed. `FilterChip` and `InputChip` exported
  legacy `checked` flags but did not stamp explicit `checked_state=true|false`, which left the
  accessibility contract weaker than the shadcn RadioGroup/Checkbox/ToggleGroup action-state gates.
  The fixed components now publish `SemanticsCheckedState::True/False` alongside `checked`.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-state-matrix-chip-action-state.json`
  starts directly on `material3_state_matrix` with `gallery-material3`, checks selected/disabled
  FilterChip and unselected InputChip semantics, drives primary and trailing-icon clicks, and
  validates `/shell/last_action` for each dispatched action.
- implementation anchors:
  `ecosystem/fret-ui-material3/src/filter_chip.rs`,
  `ecosystem/fret-ui-material3/src/input_chip.rs`,
  `ecosystem/fret-ui-material3/tests/radio_alignment.rs`,
  `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-state-matrix-chip-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-material3-state-matrix-chip-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  first checked-state failure packet:
  `target/fret-diag-material3-state-matrix-chip-action-state-v2/sessions/1779528536077-91872/1779528622600/ai.packet`;
  focused runtime AI packet after fix:
  `target/fret-diag-material3-state-matrix-chip-action-state-v4/sessions/1779529811396-91432/1779529828041/ai.packet`;
  focused runtime pack:
  `target/fret-diag-material3-state-matrix-chip-action-state-v4/sessions/1779529811396-91432/share/1779529828041.zip`;
  dedicated suite summary:
  `target/fret-diag-material3-state-matrix-chip-action-state-suite-v1/sessions/1779529969098-52044/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-state-matrix-chip-action-state.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt --package fret-ui-material3 --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-material3 chips_export_checked_state_for_selected_semantics --no-fail-fast --no-capture`
  (run id `6a401c1c-2c72-4eb9-9089-e6d87751e6b9`);
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_material3_state_matrix_chip_action_state --no-fail-fast --no-capture`
  (run id `371aa041-7561-4bc3-a8b3-f48ee8a4006e`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-material3`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-state-matrix-chip-action-state.json --dir target/fret-diag-material3-state-matrix-chip-action-state-v4 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-material3 --bin fret-ui-gallery`
  (run id `1779529828041`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-material3-state-matrix-chip-action-state --dir target/fret-diag-material3-state-matrix-chip-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-material3 --bin fret-ui-gallery`
  (script run id `1779529987189`, suite run id `1779529969098-52044`, 1/1 passed);
  `git diff --check`.

## Windowed Rows Surface Scroll Refresh

- invariant:
  the Windowed Rows Surface torture page must publish scroll offset changes and repaint scene
  fingerprints when `visible_start` changes, even though the UI is rendered as a single
  Scroll+Canvas surface instead of per-row element subtrees.
- finding:
  no mechanism defect was reproduced. The focused runtime and dedicated suite produced valid
  offset-change samples and the `visible_start` repaint post-run gate passed. The slice did close a
  diagnostics wiring gap by pointing the promoted suite and roundtrip test at the real
  `ui-gallery/windowed-rows/...` script instead of the legacy top-level redirect.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/windowed-rows/ui-gallery-windowed-rows-surface-scroll-refresh.json`
  starts directly on `windowed_rows_surface_torture`, moves the pointer onto
  `ui-gallery-windowed-rows-root`, wheels the single-node surface, and captures a bounded bundle so
  `fret-diag` can enforce offset-change and visible-start repaint post-run checks.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/torture/windowed_rows_surface_torture.rs`,
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
  `crates/fret-diag/src/diag_policy.rs`,
  `crates/fret-diag/src/stats/windowed_rows.rs`,
  `tools/diag-scripts/ui-gallery/windowed-rows/ui-gallery-windowed-rows-surface-scroll-refresh.json`,
  `tools/diag-scripts/suites/ui-gallery-windowed-rows-surface/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  preflight-only failure from prebuilt binary feature inference:
  `target/fret-diag-windowed-rows-surface-scroll-refresh-v1/sessions/1779531509683-57936`;
  focused runtime AI packet:
  `target/fret-diag-windowed-rows-surface-scroll-refresh-v2/sessions/1779531525117-82708/1779531611027/ai.packet`;
  focused runtime pack:
  `target/fret-diag-windowed-rows-surface-scroll-refresh-v2/sessions/1779531525117-82708/share/1779531611027.zip`;
  dedicated suite summary:
  `target/fret-diag-windowed-rows-surface-suite-v1/sessions/1779531680574-89080/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/windowed-rows/ui-gallery-windowed-rows-surface-scroll-refresh.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-windowed-rows-surface/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt --package fret-diag --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_windowed_rows_surface_scroll_refresh --no-fail-fast --no-capture`
  (run id `aaf9b8d8-429f-4671-bcb0-3cfc81d242b4`);
  `cargo nextest run --cargo-profile dev-fast -p fret-diag windowed_rows_gallery_surface_scroll_refresh_uses_offset_and_repaint_gates --no-fail-fast --no-capture`
  (run id `4e1e1aaf-3c6b-4187-b3ee-9bfc9395525e`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/windowed-rows/ui-gallery-windowed-rows-surface-scroll-refresh.json --dir target/fret-diag-windowed-rows-surface-scroll-refresh-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  (run id `1779531611027`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-windowed-rows-surface --dir target/fret-diag-windowed-rows-surface-suite-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness --bin fret-ui-gallery`
  (script run id `1779531695396`, suite run id `1779531680574-89080`, 1/1 passed);
  `git diff --check`.

## AI Plan Collapsible Relation Action State

- invariant:
  the AI Elements Plan trigger must keep fresh collapsible action-state and relation semantics
  through closed -> open -> closed transitions: closed triggers expose `expanded=false` and
  `invoke=true` without a resolved `controls` edge; open triggers expose `expanded=true`,
  `invoke=true`, and a `controls` edge to the rendered `PlanContent` wrapper; closing clears the
  edge again.
- finding:
  no mechanism defect was reproduced. The runtime gate confirmed the existing `fret-ui-ai`
  implementation already lines up the trigger's `controls_element` with the stable content root
  that `PlanContent::test_id(...)` decorates. This slice promoted the old screenshot smoke into a
  relation/action-state gate and gave it a dedicated suite entry.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-plan-demo-toggle.json` starts directly on
  `ai_plan_demo`, scrolls the Plan trigger into view, asserts closed trigger semantics, opens the
  content, proves `controls -> ui-ai-plan-content-marker`, captures a layout sidecar plus
  screenshot, then closes and proves the relation is empty again.
- implementation anchors:
  `ecosystem/fret-ui-ai/src/elements/plan.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/ai/plan_demo.rs`,
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-plan-demo-toggle.json`,
  `tools/diag-scripts/suites/ui-gallery-ai-plan-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-ai-plan-relation-action-state-v1/sessions/1779532816937-65624/1779532840657/ai.packet`;
  focused runtime pack:
  `target/fret-diag-ai-plan-relation-action-state-v1/sessions/1779532816937-65624/share/1779532840657.zip`;
  dedicated suite summary:
  `target/fret-diag-ai-plan-relation-action-state-suite-v1/sessions/1779532864319-95604/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-plan-demo-toggle.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-ai-plan-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_ai_plan_demo_toggle --no-fail-fast --no-capture`
  (run id `0d24e7ba-d371-45dd-8c8b-71e51c736b81`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-ai plan_trigger_stamps_controls_and_expanded_for_collapsible_semantics plan_content_uses_controller_content_id_as_root_element_id --no-fail-fast --no-capture`
  (run id `f7e03d04-067f-4a52-beb4-2852f39e5f34`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-plan-demo-toggle.json --dir target/fret-diag-ai-plan-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (run id `1779532840657`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-ai-plan-relation-action-state --dir target/fret-diag-ai-plan-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (script run id `1779532876930`, suite run id `1779532864319-95604`, 1/1 passed);
  `git diff --check`.

## AI Reasoning Collapsible Relation Action State

- invariant:
  the AI Elements Reasoning trigger must keep fresh collapsible action-state and relation semantics
  through streaming-driven closed -> open -> closed transitions: closed triggers expose
  `expanded=false`, `invoke=true`, and no resolved `controls` edge; streaming opens the content and
  mutates the trigger to `expanded=true` with `controls -> ReasoningContent`; stopping streaming
  auto-closes the content and clears the relation again.
- finding:
  a real recipe diagnostics gap was reproduced and fixed. `ReasoningTrigger` exported the correct
  expanded state and a `controls` edge, but the edge targeted the shadcn `Collapsible` internal
  motion/content wrapper while `ReasoningContent::test_id(...)` decorated an inner content node.
  Diagnostics therefore could not resolve `controls -> ui-ai-reasoning-content` even though both
  nodes existed. The fix adds a `Collapsible::content_test_id(...)` wrapper hook and makes the
  Reasoning composition API promote `ReasoningContent::test_id(...)` to that actual wrapper target.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-reasoning-demo-auto-open-close.json` starts
  directly on `ai_reasoning_demo`, asserts closed trigger semantics, starts streaming, proves
  content mount plus `expanded=true` and `controls -> ui-ai-reasoning-content`, captures a layout
  sidecar plus screenshot, stops streaming, and proves auto-close clears the relation.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/collapsible.rs`,
  `ecosystem/fret-ui-ai/src/elements/reasoning.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/ai/reasoning_demo.rs`,
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-reasoning-demo-auto-open-close.json`,
  `tools/diag-scripts/suites/ui-gallery-ai-reasoning-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  first relation failure AI packet:
  `target/fret-diag-ai-reasoning-relation-action-state-v1/sessions/1779533634109-94232/1779533669080/ai.packet`;
  focused runtime AI packet after fix:
  `target/fret-diag-ai-reasoning-relation-action-state-v2/sessions/1779534750446-89492/1779534765888/ai.packet`;
  focused runtime pack:
  `target/fret-diag-ai-reasoning-relation-action-state-v2/sessions/1779534750446-89492/share/1779534765888.zip`;
  dedicated suite summary:
  `target/fret-diag-ai-reasoning-relation-action-state-suite-v1/sessions/1779534799046-95680/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-reasoning-demo-auto-open-close.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-ai-reasoning-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt --package fret-ui-shadcn --package fret-ui-ai --package fret-ui-gallery --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_ai_reasoning_demo_auto_open_close --no-fail-fast --no-capture`
  (run id `8894e9ee-81a3-4615-9d01-71ed2dbe5d83`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-ai reasoning_children_composition_renders_content_by_default_when_streaming reasoning_content_test_id_marks_collapsible_controls_target reasoning_controller_is_available_inside_custom_parts --no-fail-fast --no-capture`
  (run id `56645f0f-fb7b-4c32-b7cd-b46ff20ac87f`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib collapsible_content_test_id_stamps_controls_target_wrapper collapsible_trigger_controls_resolves_to_content_when_open collapsible_custom_trigger_receives_expanded_semantics --no-fail-fast --no-capture`
  (run id `4d1dc26f-3179-4214-bdfc-51e74424796a`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --features gallery-dev ai_reasoning_stack_trace_and_voice_selector_use_shared_chrome_text_roles --no-fail-fast --no-capture`
  (run id `4999a68d-0566-4631-aa01-dc3373abdcf6`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-reasoning-demo-auto-open-close.json --dir target/fret-diag-ai-reasoning-relation-action-state-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (run id `1779534765888`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-ai-reasoning-relation-action-state --dir target/fret-diag-ai-reasoning-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (script run id `1779534813901`, suite run id `1779534799046-95680`, 1/1 passed);
  `git diff --check`.

## AI Task Collapsible Relation Action State

- invariant:
  the AI Elements Task trigger must keep fresh collapsible action-state and relation semantics
  through default-open -> closed -> reopened transitions: open triggers expose `expanded=true`,
  `invoke=true`, and `controls -> TaskContent`; closed triggers keep `invoke=true` but clear the
  resolved `controls` edge; reopening restores the content edge.
- finding:
  a real recipe diagnostics gap was reproduced and fixed. The old smoke clicked the Task trigger
  and captured content visibility but did not prove action-state or relation semantics. The first
  promoted probe found `ui-ai-task-demo-trigger` on an outer container with role `button` but no
  exported `invoke`, `expanded`, or `controls`. Moving the id to the pressable exposed the deeper
  contract issue: `TaskTrigger` returned a styled container as the `Collapsible` trigger root, so
  `Collapsible` stamped relation state on that wrapper instead of the pressable action root. The
  fix makes the pressable the root element and nests the visual row inside it. `TaskContent` ids
  are also promoted to the shadcn content wrapper via `Collapsible::content_test_id(...)`, matching
  the actual `controls` target.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-task-demo-toggle.json` starts directly on
  `ai_task_demo`, proves the initially open Task trigger exports `expanded=true`, `invoke=true`,
  and `controls -> ui-ai-task-demo-content`, captures a layout sidecar plus screenshot, closes the
  Task and proves the content disappears plus the relation clears, then reopens and proves the
  relation is restored.
- implementation anchors:
  `ecosystem/fret-ui-ai/src/elements/task.rs`,
  `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-task-demo-toggle.json`,
  `tools/diag-scripts/suites/ui-gallery-ai-task-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/ai/task_demo.rs`.
- evidence anchors:
  initial smoke/probe AI packet:
  `target/fret-diag-ai-task-current-smoke-probe-v1/sessions/1779535197077-26148/1779535218767/ai.packet`;
  focused runtime AI packet after fix:
  `target/fret-diag-ai-task-relation-action-state-v1/sessions/1779535809422-77104/1779535893081/ai.packet`;
  focused runtime pack:
  `target/fret-diag-ai-task-relation-action-state-v1/sessions/1779535809422-77104/share/1779535893081.zip`;
  dedicated suite summary:
  `target/fret-diag-ai-task-relation-action-state-suite-v1/sessions/1779536058135-86692/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-task-demo-toggle.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-ai-task-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt --package fret-ui-ai --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_ai_task_demo_toggle --no-fail-fast --no-capture`
  (run id `a09a00fe-7083-4c3a-b4b8-c06a226f7141fa7`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-ai task_compound_children_surface_resolves_trigger_and_content task_trigger_and_content_test_ids_mark_collapsible_relation_endpoints task_trigger_default_row_attaches_foreground_without_wrapper task_surfaces_use_shared_typography_presets --no-fail-fast --no-capture`
  (run id `9b6eb6f4-74fa-4b98-b083-06a7263b772d`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --features gallery-dev ai_task_demo_uses_shared_content_text_roles --no-fail-fast --no-capture`
  (run id `f293fbae-da9f-44ae-a40b-f2da894e4aba`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-task-demo-toggle.json --dir target/fret-diag-ai-task-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (run id `1779535893081`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-ai-task-relation-action-state --dir target/fret-diag-ai-task-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (script run id `1779536079556`, suite run id `1779536058135-86692`, 1/1 passed).

## Collapsible Basic Relation Action State

- invariant:
  the compact shadcn Collapsible Basic trigger must keep fresh action-state and relation semantics
  through closed -> open -> closed -> reopened transitions: closed triggers expose
  `expanded=false`, `invoke=true`, and no resolved `controls` edge; opening exposes
  `expanded=true`, `invoke=true`, and `controls -> content wrapper`; closing clears the edge; and
  reopening restores it.
- finding:
  a real diagnostics endpoint gap was reproduced and fixed. The previous Basic script only checked
  bounds after repeated clicks, so it could pass without proving trigger action-state or relation
  semantics. The first probe also showed `ui-gallery-collapsible-basic-content` was ambiguous: one
  match was the docs `DocSection` content wrapper and another was the inner Collapsible content
  node, while trigger `controls` pointed at the internal shadcn motion/content wrapper. The fix
  avoids the generated `*-content` naming collision and stamps
  `ui-gallery-collapsible-basic-panel` on the actual controls target via
  `Collapsible::content_test_id(...)`.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-basic-double-click-close.json`
  starts directly on `collapsible`, scrolls Basic into view, asserts the initial closed
  `expanded=false`, `invoke=true`, and empty `controls` state, opens and proves
  `controls -> ui-gallery-collapsible-basic-panel`, captures a layout sidecar plus screenshot,
  closes and proves the relation clears, then reopens and proves the relation returns.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/collapsible/basic.rs`,
  `apps/fret-ui-gallery/tests/collapsible_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-basic-double-click-close.json`,
  `tools/diag-scripts/suites/ui-gallery-collapsible-basic-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`,
  `ecosystem/fret-ui-shadcn/src/collapsible.rs`.
- evidence anchors:
  initial smoke/probe AI packet:
  `target/fret-diag-collapsible-basic-current-smoke-probe-v1/sessions/1779536818310-2952/1779536832035/ai.packet`;
  focused runtime AI packet after fix:
  `target/fret-diag-collapsible-basic-relation-action-state-v1/sessions/1779537310318-97472/1779537328201/ai.packet`;
  focused runtime pack:
  `target/fret-diag-collapsible-basic-relation-action-state-v1/sessions/1779537310318-97472/share/1779537328201.zip`;
  focused final capture bundle:
  `target/fret-diag-collapsible-basic-relation-action-state-v1/sessions/1779537310318-97472/1779537449506-ui-gallery-collapsible-basic-double-click-close/bundle.json`;
  dedicated suite summary:
  `target/fret-diag-collapsible-basic-relation-action-state-suite-v1/sessions/1779537469123-97920/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-basic-double-click-close.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-collapsible-basic-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo fmt --package fret-ui-gallery --package fret-diag-protocol`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_collapsible_basic_double_click_close --no-fail-fast --no-capture`
  (run id `6fe0d779-8098-4c02-93bb-b71f3973f23c`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --features gallery-dev collapsible_docs_path_snippets_stay_copyable_and_docs_aligned collapsible_docs_diag_scripts_cover_docs_smoke_and_existing_notes_follow_ups --no-fail-fast --no-capture`
  (run id `a7f2c102-95f8-450d-b511-ea7ba941d931`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-basic-double-click-close.json --dir target/fret-diag-collapsible-basic-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (run id `1779537328201`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-collapsible-basic-relation-action-state --dir target/fret-diag-collapsible-basic-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (script run id `1779537488552`, suite run id `1779537469123-97920`, 1/1 passed).

## Tooltip Focus Relation Action State

- invariant:
  the keyboard-focus Tooltip trigger must keep fresh focus/action and relation semantics through
  closed -> open -> closed transitions: closed triggers expose `focus=true`, `invoke=true`, and no
  resolved `described_by` edge; tab focus opens the tooltip and adds
  `described_by -> ui-gallery-tooltip-focus-content-node`; Escape closes and clears the relation
  again.
- finding:
  diagnostics endpoint drift was reproduced and fixed. `ui-gallery-tooltip-focus-panel` is the
  geometry-only popper marker, while `described_by` targets the actual `role=tooltip` content
  node. `TooltipContent::test_id(...)` now stamps that semantics node so the runtime gate can
  assert the true relation target.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-focus-opens.json` starts directly on
  tooltip, scrolls `ui-gallery-tooltip-focus-start` into view, clicks the focus starter, tabs to
  the trigger, proves focus/invoke, waits for `ui-gallery-tooltip-focus-content-node`,
  `ui-gallery-tooltip-focus-panel`, and `ui-gallery-tooltip-focus-arrow`, asserts
  `described_by -> ui-gallery-tooltip-focus-content-node`, captures a layout sidecar plus
  screenshot, presses Escape, and proves the relation clears.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/tooltip.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/tooltip/keyboard_focus.rs`,
  `apps/fret-ui-gallery/tests/tooltip_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-focus-opens.json`,
  `tools/diag-scripts/suites/ui-gallery-tooltip-focus-relation/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  initial probe AI packet:
  `target/fret-diag-tooltip-focus-current-probe-v1/sessions/1779539319670-97768/1779539334280/ai.packet`;
  focused runtime AI packet:
  `target/fret-diag-tooltip-focus-relation-v1/sessions/1779542537375-93968/1779542563339/ai.packet`;
  focused runtime pack:
  `target/fret-diag-tooltip-focus-relation-v1/sessions/1779542537375-93968/share/1779542563339.zip`;
  focused runtime bundle:
  `target/fret-diag-tooltip-focus-relation-v1/sessions/1779542537375-93968/1779542563339/bundle.json`;
  dedicated suite summary:
  `target/fret-diag-tooltip-focus-relation-suite-v2/sessions/1779546745483-87812/suite.summary.json`.
- run results:
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_tooltip_focus_opens --no-fail-fast --no-capture`
  (run id `2cc3ca43-3778-4e41-812b-bcab83dd8daf`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --features gallery-dev tooltip_page_documents_source_axes_and_children_api_decision tooltip_snippets_stay_copyable_and_docs_aligned tooltip_docs_diag_scripts_cover_docs_path_and_follow_ups --no-fail-fast --no-capture`
  (run id `890f9914-550e-4d90-8cba-671e6c3a4878`);
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-tooltip-focus-relation --dir target/fret-diag-tooltip-focus-relation-suite-v2 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery`
  (script run id `1779546771353`, suite run id `1779546745483-87812`, 1/1 passed);
  `python tools/check_diag_scripts_registry.py`
  - result: passed.

## AlertDialog Demo Relation Action State

- invariant:
  the shadcn AlertDialog Demo trigger/content pair must keep fresh modal relation and action-state
  semantics through closed -> modal open -> closed transitions: closed triggers expose
  `expanded=false`, `invoke=true`, and no resolved `controls` edge; opening installs modal/focus
  barrier roots, keeps relation endpoints resolvable through `controls -> AlertDialogContent`,
  exposes content `labelled_by` and `described_by` edges to the stable title/description nodes, and
  keeps Cancel/Action invokable; closing restores focus to the trigger, clears barrier roots, resets
  `expanded=false`, and clears the `controls` edge.
- finding:
  no AlertDialog component defect was reproduced. The first focused run found a diagnostics
  authoring pitfall: once the modal barrier is active, ordinary selectors correctly treat the
  underlay trigger as inert, so `expanded_is` on the trigger cannot be used to prove open-state
  relation semantics. The script now asserts active modal/focus barrier roots and uses relation
  endpoint resolution for the open trigger `controls` edge. A second run confirmed the newly added
  title/description test ids required rebuilding `target/dev-fast/fret-ui-gallery.exe`; after the
  rebuild the focused gate passed.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/alert-dialog/ui-gallery-alert-dialog-demo-relation-action-state.json`
  starts directly on `alert_dialog`, scrolls the Demo trigger into view, asserts closed trigger
  action/relation state, opens the dialog, proves barrier roots, content role, trigger/content and
  title/description relation endpoints, Cancel/Action invoke exposure, captures a layout sidecar
  plus screenshot, closes with Cancel, and proves focus restore plus teardown semantics.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/demo.rs`,
  `tools/diag-scripts/ui-gallery/alert-dialog/ui-gallery-alert-dialog-demo-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-alert-dialog-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  initial modal-underlay selector failure AI packet:
  `target/fret-diag-alert-dialog-relation-action-state-v1/sessions/1779562755425-105832/1779562768730/ai.packet`;
  stale-binary title/description selector failure AI packet:
  `target/fret-diag-alert-dialog-relation-action-state-v2/sessions/1779563462670-4272/1779563476042/ai.packet`;
  focused runtime AI packet after rebuild:
  `target/fret-diag-alert-dialog-relation-action-state-v3/sessions/1779563774152-109180/1779563787191/ai.packet`;
  focused runtime pack:
  `target/fret-diag-alert-dialog-relation-action-state-v3/sessions/1779563774152-109180/share/1779563787191.zip`;
  dedicated suite summary:
  `target/fret-diag-alert-dialog-relation-action-state-suite-v1/sessions/1779563883496-110116/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/alert-dialog/ui-gallery-alert-dialog-demo-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-alert-dialog-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_alert_dialog_demo_relation_action_state --no-fail-fast --no-capture`
  (run id `8c83fe42-96c8-4bd1-8169-b1a429d79cda`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/alert-dialog/ui-gallery-alert-dialog-demo-relation-action-state.json --dir target/fret-diag-alert-dialog-relation-action-state-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779563787191`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-alert-dialog-relation-action-state --dir target/fret-diag-alert-dialog-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779563896213`, suite run id `1779563883496-110116`, 1/1 passed).

## Sheet Demo Relation Action State

- invariant:
  the shadcn Sheet Demo trigger/content pair must keep fresh modal relation and action-state
  semantics through closed -> modal open -> closed transitions: closed triggers expose
  `expanded=false`, `invoke=true`, and no resolved `controls` edge; opening installs modal/focus
  barrier roots, keeps relation endpoints resolvable through `controls -> SheetContent`, exposes
  content `labelled_by` and `described_by` edges to the stable title/description nodes, focuses the
  first editable input, and keeps input/save/close actions available; closing restores focus to the
  trigger, clears barrier roots, resets `expanded=false`, and clears the `controls` edge.
- finding:
  a real Sheet recipe semantics gap was reproduced and fixed. Unlike Dialog/AlertDialog, Sheet did
  not stamp Dialog-style trigger `expanded/controls` metadata and `SheetContent` did not participate
  in the modal title/description registry. The fix reuses `radix_dialog::apply_dialog_trigger_a11y`
  from the Sheet recipe, stores the last content element for the closed frame, wraps Sheet content
  construction in the modal a11y scope, and registers `SheetTitle`/`SheetDescription` as modal
  relation endpoints. The suite pass also caught an authoring issue: the initial title/description
  test ids collided with DocSection-generated ids, so the open panel now uses
  `ui-gallery-sheet-demo-dialog-title` and `ui-gallery-sheet-demo-dialog-description`.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/sheet/ui-gallery-sheet-demo-relation-action-state.json` starts
  directly on `sheet`, scrolls the Demo trigger into view, asserts closed trigger action/relation
  state, opens the sheet, proves barrier roots, content role, trigger/content and title/description
  relation endpoints, input/save/close action exposure, captures a layout sidecar plus screenshot,
  closes through the footer close action, and proves focus restore plus teardown semantics.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/sheet.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/sheet/demo.rs`,
  `tools/diag-scripts/ui-gallery/sheet/ui-gallery-sheet-demo-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-sheet-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  initial trigger action endpoint failure AI packet:
  `target/fret-diag-sheet-relation-action-state-v1/sessions/1779566147348-14700/1779566159272/ai.packet`;
  focused runtime AI packet after Sheet a11y fix and stable selector cleanup:
  `target/fret-diag-sheet-relation-action-state-v4/sessions/1779566903805-109808/1779566915448/ai.packet`;
  focused runtime pack:
  `target/fret-diag-sheet-relation-action-state-v4/sessions/1779566903805-109808/share/1779566915448.zip`;
  dedicated suite summary:
  `target/fret-diag-sheet-relation-action-state-suite-v2/sessions/1779566998050-99572/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/sheet/ui-gallery-sheet-demo-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-sheet-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/sheet/demo.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs ecosystem/fret-ui-shadcn/src/sheet.rs`;
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib sheet_children_builder_exports_trigger_and_content_relations -- --nocapture`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_sheet_demo_relation_action_state --no-fail-fast --no-capture`
  (run id `4453ab41-3619-41a9-b9ed-b1a034c4d782`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/sheet/ui-gallery-sheet-demo-relation-action-state.json --dir target/fret-diag-sheet-relation-action-state-v4 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779566915448`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-sheet-relation-action-state --dir target/fret-diag-sheet-relation-action-state-suite-v2 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779567009477`, suite run id `1779566998050-99572`, 1/1 passed).

## Drawer Demo Relation Action State

- invariant:
  the shadcn Drawer Demo trigger/content pair must keep fresh modal relation and action-state
  semantics through closed -> modal open -> closed transitions: closed triggers expose
  `expanded=false`, `invoke=true`, and no resolved `controls` edge; opening installs modal/focus
  barrier roots, keeps relation endpoints resolvable through `controls -> DrawerContent`, exposes
  content `labelled_by` and `described_by` edges to the stable title/description nodes, and keeps
  drawer-local actions invokable; closing restores focus to the trigger, clears barrier roots,
  resets `expanded=false`, and clears the `controls` edge.
- finding:
  a real Sheet-backed Drawer semantics gap was reproduced and fixed. Drawer delegates modal root
  behavior to Sheet, but Drawer adds a drag/motion wrapper around its content. Sheet's generic
  trigger a11y path used the returned content root as the `controls` target, which could point at
  that wrapper instead of the actual `role=dialog` DrawerContent node. DrawerContent also stamped
  only `role=dialog`, so it did not read the modal title/description registry populated by
  `DrawerTitle` and `DrawerDescription`. The fix makes Sheet prefer the first returned content-tree
  element with `SemanticsRole::Dialog` as the trigger `controls` target, preserving the fallback to
  the returned root, and makes DrawerContent attach modal `labelled_by` / `described_by` relations.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-demo-relation-action-state.json` starts
  directly on `drawer`, scrolls the Demo trigger into view, asserts closed trigger action/relation
  state, opens the drawer, proves barrier roots, content role, trigger/content and
  title/description relation endpoints, decrease/increase/submit/cancel action exposure, captures
  a layout sidecar plus screenshot, closes through the footer close action, and proves focus
  restore plus teardown semantics.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/sheet.rs`,
  `ecosystem/fret-ui-shadcn/src/drawer.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/drawer/demo.rs`,
  `tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-demo-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-drawer-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  initial focused unit failure:
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib drawer_children_builder_exports_trigger_and_content_relations -- --nocapture`
  failed because the Drawer trigger `controls` edge did not include the DrawerContent node;
  focused runtime AI packet after fix:
  `target/fret-diag-drawer-relation-action-state-v1/sessions/1779568067455-77948/1779568082318/ai.packet`;
  focused runtime pack:
  `target/fret-diag-drawer-relation-action-state-v1/sessions/1779568067455-77948/share/1779568082318.zip`;
  dedicated suite summary:
  `target/fret-diag-drawer-relation-action-state-suite-v1/sessions/1779568174460-103024/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-demo-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-drawer-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/drawer/demo.rs ecosystem/fret-ui-shadcn/src/drawer.rs ecosystem/fret-ui-shadcn/src/sheet.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib drawer_children_builder_exports_trigger_and_content_relations -- --nocapture`;
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib sheet_children_builder_exports_trigger_and_content_relations -- --nocapture`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_drawer_demo_relation_action_state --no-fail-fast --no-capture`
  (run id `6fc23ee4-4488-423e-9037-e0a466bf9809`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-demo-relation-action-state.json --dir target/fret-diag-drawer-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779568082318`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-drawer-relation-action-state --dir target/fret-diag-drawer-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779568191247`, suite run id `1779568174460-103024`, 1/1 passed);
  `git diff --check -- apps/fret-ui-gallery/src/ui/snippets/drawer/demo.rs ecosystem/fret-ui-shadcn/src/drawer.rs ecosystem/fret-ui-shadcn/src/sheet.rs tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-demo-relation-action-state.json tools/diag-scripts/suites/ui-gallery-drawer-relation-action-state/suite.json crates/fret-diag-protocol/tests/script_json_roundtrip.rs tools/diag-scripts/index.json`
  - result: passed.
## Dialog Demo Relation Action State

- invariant:
  the shadcn Dialog Demo trigger/content pair must keep fresh modal relation and action-state
  semantics through closed -> modal open -> closed transitions: closed triggers expose
  `expanded=false`, `invoke=true`, and no resolved `controls` edge; opening installs modal/focus
  barrier roots, keeps relation endpoints resolvable through `controls -> DialogContent`, exposes
  content `labelled_by` and `described_by` edges to the stable title/description nodes, keeps both
  inputs invokable for value changes, and closing restores focus to the trigger, clears barrier
  roots, resets `expanded=false`, and clears the `controls` edge.
- finding:
  no Dialog implementation defect was reproduced. The run only promoted the existing Dialog
  mechanism into a durable UI Gallery gate and suite.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-demo-relation-action-state.json` starts
  directly on `dialog`, scrolls the Demo trigger into view, asserts closed trigger action/relation
  state, opens the dialog, proves barrier roots, content role, trigger/content and
  title/description relation endpoints, input/save/cancel action exposure, captures a layout
  sidecar plus screenshot, closes through Cancel, and proves focus restore plus teardown semantics.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/dialog.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/dialog/demo.rs`,
  `tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-demo-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-dialog-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-dialog-relation-action-state-v1/sessions/1779569173628-108412/1779569186080/ai.packet`;
  focused runtime pack:
  `target/fret-diag-dialog-relation-action-state-v1/sessions/1779569173628-108412/share/1779569186080.zip`;
  dedicated suite summary:
  `target/fret-diag-dialog-relation-action-state-suite-v1/sessions/1779569276359-76248/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-demo-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-dialog-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_dialog_demo_relation_action_state --no-fail-fast --no-capture`
  (run id `f24a4ab6-2fb3-4c83-b5a2-bba51da0eca1`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-demo-relation-action-state.json --dir target/fret-diag-dialog-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779569186080`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-dialog-relation-action-state --dir target/fret-diag-dialog-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779569288674`, suite run id `1779569276359-76248`, 1/1 passed).

## NavigationMenu Docs Demo Relation Action State

- invariant:
  the shadcn NavigationMenu docs-demo Components trigger/content pair must keep fresh relation and
  action-state semantics through closed -> open -> keyboard-entry -> closed transitions: the
  closed trigger exposes `role=button`, `expanded=false`, and `invoke=true`; contentless top-level
  links stay link-like and publish no dangling `controls` edge; opening the Components item exposes
  `expanded=true` and a resolvable `controls` edge to the actual active content wrapper; content
  links keep `role=link` and `invoke=true`; ArrowDown from the focused trigger enters the first
  content link; Escape unmounts the viewport/content and returns the trigger to `expanded=false`.
- finding:
  a recipe diagnostics endpoint gap was found and fixed. The NavigationMenu primitive already
  computes a stable internal viewport-content wrapper id so triggers can publish `controls` before
  the viewport mounts, but the shadcn recipe exposed only `trigger_test_id` and `viewport_test_id`.
  That meant a runtime script could prove the viewport opened, but could not name the actual
  wrapper that the trigger controlled. `NavigationMenuItem::content_test_id(...)` now stamps the
  active content wrapper's `PressableA11y.test_id`; the UI Gallery docs demo stamps the Components
  endpoint as `ui-gallery-navigation-menu-docs-demo-content-components`. No runtime interaction
  defect was reproduced after that endpoint was made observable.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-demo-relation-action-state.json`
  starts directly on `navigation_menu`, scrolls the docs demo into view, asserts closed trigger
  and contentless-link semantics, opens the Components viewport, proves trigger/content relation
  resolution plus link action state, moves focus into the first content link, captures a layout
  sidecar plus screenshot, closes with Escape, and proves teardown semantics.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs`,
  `tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-demo-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-navigation-menu-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-navigation-menu-relation-action-state-v1/sessions/1779572326270-109576/1779572337827/ai.packet`;
  focused runtime pack:
  `target/fret-diag-navigation-menu-relation-action-state-v1/sessions/1779572326270-109576/share/1779572337827.zip`;
  dedicated suite summary:
  `target/fret-diag-navigation-menu-relation-action-state-suite-v1/sessions/1779572364960-84260/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-demo-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-navigation-menu-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check ecosystem/fret-ui-shadcn/src/navigation_menu.rs apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib navigation_menu_content_test_id_stamps_controls_target_wrapper -- --nocapture`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_navigation_menu_docs_demo_relation_action_state --no-fail-fast --no-capture`
  (run id `ad62a166-01ff-45d3-bdb7-c92fdbd683a3`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-demo-relation-action-state.json --dir target/fret-diag-navigation-menu-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779572337827`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-navigation-menu-relation-action-state --dir target/fret-diag-navigation-menu-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779572376459`, suite run id `1779572364960-84260`, 1/1 passed).

## Popover Demo Relation Action State

- invariant:
  the shadcn Popover Demo trigger/content pair must keep fresh non-modal relation and
  action-state semantics through closed -> open -> Escape-closed transitions: the closed trigger
  exposes `role=button`, `expanded=false`, `invoke=true`, and no resolved `controls` edge; opening
  exposes a resolvable `controls` edge to the Popover dialog wrapper, keeps the visual panel as a
  separate `role=panel` node, installs no modal or focus barrier roots, and keeps all dimensions
  inputs writable; Escape closes the popover, restores focus to the trigger, resets
  `expanded=false`, and clears the `controls` edge.
- finding:
  no Popover interaction defect was reproduced. The first focused probe found a diagnostics
  selector assumption: the semantic trigger endpoint is the Popover root test id
  `ui-gallery-popover-demo-popover`, while the authored Button test id lands on visual
  `.chrome`/label descendants. The first suite run then exposed a Gallery duplicate-test-id
  hygiene defect: DocSection already owns `ui-gallery-popover-demo-title` and
  `ui-gallery-popover-demo-description`, so the overlay header stopped reusing those IDs.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-demo-relation-action-state.json`
  starts directly on `popover`, scrolls the Demo section into view, asserts closed trigger
  action/relation state, opens the popover, proves dialog-wrapper and panel roles, proves
  trigger/content relation resolution, verifies non-modal barrier absence and input action state,
  captures a layout sidecar plus screenshot, closes with Escape, and proves focus restore plus
  teardown semantics.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/popover.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/popover/demo.rs`,
  `tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-demo-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-popover-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  initial focused selector failure:
  `target/fret-diag-popover-relation-action-state-v1/sessions/1779573406048-106528/1779573419118/ai.packet`;
  duplicate-id suite lint failure before Gallery cleanup:
  `target/fret-diag-popover-relation-action-state-suite-v1/sessions/1779573703268-103052/1779573793718-ui-gallery-popover-demo-open-relation-action-state/check.lint.json`;
  focused runtime AI packet after cleanup:
  `target/fret-diag-popover-relation-action-state-v3/sessions/1779573924306-108720/1779573937811/ai.packet`;
  focused runtime pack:
  `target/fret-diag-popover-relation-action-state-v3/sessions/1779573924306-108720/share/1779573937811.zip`;
  dedicated suite summary:
  `target/fret-diag-popover-relation-action-state-suite-v2/sessions/1779574027661-101288/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-demo-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-popover-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/popover/demo.rs ecosystem/fret-ui-shadcn/src/popover.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib popover_trigger_exposes_expanded_and_controls_semantics -- --nocapture`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_popover_demo_relation_action_state --no-fail-fast --no-capture`
  (run id `da3ab815-3732-42ee-b3b4-c194b97f8acf`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-demo-relation-action-state.json --dir target/fret-diag-popover-relation-action-state-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779573937811`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-popover-relation-action-state --dir target/fret-diag-popover-relation-action-state-suite-v2 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779574040099`, suite run id `1779574027661-101288`, 1/1 passed).

## Avatar Dropdown Relation Action State

- invariant:
  the Avatar Dropdown composed example must preserve the authored Button as the DropdownMenu
  trigger while keeping the nested Avatar presentational: the closed trigger exposes
  `role=button`, `expanded=false`, `invoke=true`, and no resolved `controls` edge; the nested
  Avatar leaf stays `role=generic` with `invoke=false`; opening the menu exposes a resolvable
  `controls -> DropdownMenuContent` edge, focuses the `role=menu`, and keeps menu items invokable;
  Escape closes the menu, restores focus to the trigger, resets `expanded=false`, and clears the
  `controls` edge.
- finding:
  no DropdownMenu or Avatar behavior defect was reproduced. This slice added stable Gallery
  endpoints so the composed consumer path can be observed. The first focused script found a
  diagnostics authoring pitfall: `DocSection::test_id_prefix("ui-gallery-avatar-dropdown")`
  produces `ui-gallery-avatar-dropdown-content`, not a root node named
  `ui-gallery-avatar-dropdown`. The second focused script found an oracle mismatch: DropdownMenu
  should not be asserted as a Dialog/Sheet-style modal barrier-root surface. The correct gate is
  relation/action-state plus focused `role=menu` semantics and Escape teardown.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-relation-action-state.json`
  starts directly on `avatar` / `Dropdown`, asserts the section content, closed trigger state, and
  nested Avatar leaf semantics, opens the menu, proves trigger/menu relation resolution, menu focus,
  and menu-item action state, captures a layout sidecar plus screenshot, closes with Escape, and
  proves focus restore plus relation teardown.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/avatar/dropdown.rs`,
  `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`,
  `tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-avatar-dropdown-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  initial section-root authoring failure:
  `target/fret-diag-avatar-dropdown-relation-action-state-v1/sessions/1779575254174-61912/1779575262932/ai.packet`;
  barrier-root oracle mismatch before narrowing the gate:
  `target/fret-diag-avatar-dropdown-relation-action-state-v2/sessions/1779575486465-78436/1779575495118/ai.packet`;
  focused runtime AI packet:
  `target/fret-diag-avatar-dropdown-relation-action-state-v3/sessions/1779575679508-106740/1779575688231/ai.packet`;
  focused runtime pack:
  `target/fret-diag-avatar-dropdown-relation-action-state-v3/sessions/1779575679508-106740/share/1779575688231.zip`;
  dedicated suite summary:
  `target/fret-diag-avatar-dropdown-relation-action-state-suite-v1/sessions/1779575714167-95056/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-avatar-dropdown-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/avatar/dropdown.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib dropdown_menu_part_trigger_keeps_authored_button_semantics_when_avatar_is_nested_child -- --nocapture`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_avatar_dropdown_relation_action_state --no-fail-fast --no-capture`
  (run id `3e3553c4-e444-40a1-9b87-240f758071b1`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-relation-action-state.json --dir target/fret-diag-avatar-dropdown-relation-action-state-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779575688231`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-avatar-dropdown-relation-action-state --dir target/fret-diag-avatar-dropdown-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779575722734`, suite run id `1779575714167-95056`, 1/1 passed).

## Input Group Dropdown Relation Action State

- invariant:
  the Input Group Dropdown composed example must keep the text-field control and inline-end
  DropdownMenu trigger semantically separate inside the same input-group chrome: the control keeps
  `role=text_field`, focus, and `set_value`; the addon `InputGroupButton` owns `role=button`,
  `expanded`, `invoke`, and `controls`; opening the menu focuses the `role=menu`, resolves the
  trigger `controls` edge to the menu content, and keeps menu items invokable; Escape closes the
  menu, restores focus to the trigger, resets `expanded=false`, clears `controls`, and leaves the
  text field writable.
- finding:
  no InputGroup or DropdownMenu behavior defect was reproduced. The slice only added stable
  Gallery endpoints for the leading menu content and its items, then promoted the existing
  composition into a durable relation/action-state runtime gate.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-dropdown-relation-action-state.json`
  starts directly on `input_group` / `Dropdown`, asserts the section content, text-field control
  role/action state, closed addon trigger state, and empty trigger `controls`, focuses the control
  first, opens the menu from the addon trigger, proves menu role/focus, trigger/menu relation
  resolution, and item action state, captures a layout sidecar plus screenshot, closes with Escape,
  and proves focus restore plus relation teardown and post-close control writability.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/input_group/dropdown.rs`,
  `ecosystem/fret-ui-shadcn/src/input_group.rs`,
  `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`,
  `tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-dropdown-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-input-group-dropdown-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-input-group-dropdown-relation-action-state-v1/sessions/1779576481503-79512/1779576490520/ai.packet`;
  focused runtime pack:
  `target/fret-diag-input-group-dropdown-relation-action-state-v1/sessions/1779576481503-79512/share/1779576490520.zip`;
  dedicated suite summary:
  `target/fret-diag-input-group-dropdown-relation-action-state-suite-v1/sessions/1779576510828-110644/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-dropdown-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-input-group-dropdown-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/input_group/dropdown.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_input_group_dropdown_relation_action_state --no-fail-fast --no-capture`
  (run id `233b0ba6-2681-4586-ab42-721f73ca6596`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-dropdown-relation-action-state.json --dir target/fret-diag-input-group-dropdown-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779576490520`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-input-group-dropdown-relation-action-state --dir target/fret-diag-input-group-dropdown-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779576520084`, suite run id `1779576510828-110644`, 1/1 passed).

## Breadcrumb Demo Ellipsis Relation Action State

- invariant:
  the Breadcrumb Demo ellipsis dropdown must keep the collapsed breadcrumb affordance as a real
  DropdownMenu trigger: the closed ellipsis trigger exposes `role=button`, `expanded=false`,
  `invoke=true`, and no resolved `controls` edge; opening focuses the `role=menu` content, resolves
  `controls -> DropdownMenuContent`, and keeps menu items invokable; Escape closes the menu,
  restores focus to the trigger, resets `expanded=false`, and clears the `controls` edge.
- finding:
  no Breadcrumb or DropdownMenu behavior defect was reproduced. The slice added a stable
  `test_id_prefix(...)` for the Demo dropdown content and stable test ids for the remaining menu
  rows, then promoted the existing open/close smoke path into a durable relation/action-state gate.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-demo-ellipsis-relation-action-state.json`
  starts directly on `breadcrumb` / `Demo`, asserts the closed ellipsis trigger role/action/relation
  state, opens the menu, proves menu role/focus, trigger/menu relation resolution, and item action
  state, captures a layout sidecar plus screenshot, closes with Escape, and proves focus restore
  plus relation teardown.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/demo.rs`,
  `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`,
  `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`,
  `tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-demo-ellipsis-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-breadcrumb-ellipsis-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-breadcrumb-ellipsis-relation-action-state-v1/sessions/1779576942614-110248/1779576951600/ai.packet`;
  focused runtime pack:
  `target/fret-diag-breadcrumb-ellipsis-relation-action-state-v1/sessions/1779576942614-110248/share/1779576951600.zip`;
  dedicated suite summary:
  `target/fret-diag-breadcrumb-ellipsis-relation-action-state-suite-v1/sessions/1779576971606-108904/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-demo-ellipsis-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-breadcrumb-ellipsis-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/breadcrumb/demo.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_breadcrumb_demo_ellipsis_relation_action_state --no-fail-fast --no-capture`
  (run id `626eeb24-1d33-41f9-a477-f7af7bf28f8c`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-demo-ellipsis-relation-action-state.json --dir target/fret-diag-breadcrumb-ellipsis-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779576951600`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-breadcrumb-ellipsis-relation-action-state --dir target/fret-diag-breadcrumb-ellipsis-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779576980821`, suite run id `1779576971606-108904`, 1/1 passed).

## Sidebar AppSidebar Dropdown Relation Action State

- invariant:
  the Sidebar AppSidebar team switcher and account menu must behave as independent DropdownMenu
  instances in one composed Sidebar consumer: each closed trigger owns `role=button`,
  `expanded=false`, focus/invoke actions, and an empty `controls` edge; opening either menu focuses
  its `role=menu` content, resolves trigger `controls` to that menu content, and keeps menu items
  invokable; selecting a team closes the team menu and clears its relation; Escape closes the user
  menu, restores focus to the current user trigger, resets `expanded=false`, and clears `controls`.
- finding:
  the first complete focused script found a real shared non-modal overlay focus-restore defect.
  After the team menu closed and the user menu was opened by pointer, pressing Escape closed the
  user menu but focus restored to the earlier team trigger. The stale restoration came from the
  previous overlay's hidden finalizer: close-edge autofocus had already been handled, but the
  hidden finalizer still ran a second restore path and stole focus from the later dropdown.
- fix:
  `finalize_hidden_non_modal_overlay` now skips the hidden-finalizer autofocus restore when
  `close_auto_focus_handled` is already true. This keeps the close edge as the single autofocus
  owner for that overlay and prevents older hidden overlays from overwriting a newer overlay's
  correct restore target.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-app-sidebar-dropdown-relation-action-state.json`
  starts directly on `sidebar` / `AppSidebar`, asserts initial closed trigger relation/action
  state, opens the team menu, proves menu focus, `controls` resolution, and item action state,
  selects a team item and proves close/teardown, then opens the user menu, proves relation/action
  state again, closes with Escape, and proves focus returns to the user trigger. The initial closed
  relation checks use `wait_until` after content non-existence so startup snapshots can converge.
  The script intentionally avoids visible text `value_contains` assertions because default
  diagnostics launch redacts text.
- implementation anchors:
  `ecosystem/fret-ui-kit/src/window_overlays/render.rs`,
  `ecosystem/fret-ui-shadcn/tests/dropdown_menu_escape_dismiss_focus_restore.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/sidebar/app_sidebar.rs`,
  `tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-app-sidebar-dropdown-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-sidebar-dropdown-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  failing pre-fix focused bundle:
  `target/fret-diag-sidebar-dropdown-relation-action-state-v3/sessions/1779579417831-70260/1779579569845-script-step-0047-wait_until-timeout`;
  focused runtime AI packet after fix:
  `target/fret-diag-sidebar-dropdown-relation-action-state-v4/sessions/1779580492724-113160/1779580505188/ai.packet`;
  focused runtime pack after fix:
  `target/fret-diag-sidebar-dropdown-relation-action-state-v4/sessions/1779580492724-113160/share/1779580505188.zip`;
  dedicated suite summary after fix:
  `target/fret-diag-sidebar-dropdown-relation-action-state-suite-v2/sessions/1779580605772-110628/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-app-sidebar-dropdown-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-sidebar-dropdown-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/sidebar/app_sidebar.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs ecosystem/fret-ui-kit/src/window_overlays/render.rs ecosystem/fret-ui-shadcn/tests/dropdown_menu_escape_dismiss_focus_restore.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_sidebar_app_sidebar_dropdown_relation_action_state --no-fail-fast --no-capture`
  (run id `d116320b-fa71-4487-af39-67454a71f660`);
  `cargo test --profile dev-fast -p fret-ui-shadcn --test dropdown_menu_escape_dismiss_focus_restore -- --nocapture`
  (2/2 passed);
  `cargo test --profile dev-fast -p fret-ui-kit --lib non_modal_overlay -- --nocapture`
  (14/14 passed);
  `cargo test --profile dev-fast -p fret-ui-kit --lib hidden_popover -- --nocapture`
  (1/1 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/sidebar/ui-gallery-sidebar-app-sidebar-dropdown-relation-action-state.json --dir target/fret-diag-sidebar-dropdown-relation-action-state-v4 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779580505188`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-sidebar-dropdown-relation-action-state --dir target/fret-diag-sidebar-dropdown-relation-action-state-suite-v2 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779580618006`, suite run id `1779580605772-110628`, 1/1 passed).

## Pagination Demo Action Selected State

- invariant:
  the Pagination Demo must expose link-like pagination endpoints with explicit action and selected
  semantics: the root is a labelled `region`, Previous/Next and page links are `role=link`, page 2
  is the only selected page in the static demo, clickable endpoints expose `invoke=true`, and
  pointer activation dispatches the authored app commands without mutating the static selected
  example.
- finding:
  the first focused runtime pass found a diagnostics role-vocabulary gap: `SemanticsRole::Region`
  existed in the semantics tree but exported as `unknown` through diagnostics selectors, so
  `role_is region` could not be asserted. The second focused pass found an app-driver diagnostics
  gap: `CMD_APP_OPEN` and `CMD_APP_SAVE` updated `/shell/last_action`, but did not record
  `handled_by_driver=true` command dispatch decisions, so trace-based action gates could not prove
  driver ownership for those commands.
- fix:
  diagnostics selector role labels now round-trip `region`, and the Gallery driver now records
  driver-handled command dispatch decisions for `CMD_APP_OPEN` / `CMD_APP_SAVE` after updating
  `last_action`.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-demo-action-selected-state.json`
  starts directly on `pagination` / `Demo`, asserts root/link roles, selected-state, invoke action
  state, command-dispatch trace ownership for page 1 and page 2 clicks, `/shell/last_action`
  updates to `cmd.open` / `cmd.save`, and captures layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/pagination/demo.rs`,
  `ecosystem/fret-ui-shadcn/src/pagination.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/selector.rs`,
  `apps/fret-ui-gallery/src/driver/runtime_driver.rs`,
  `tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-demo-action-selected-state.json`,
  `tools/diag-scripts/suites/ui-gallery-pagination-action-selected-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  failing pre-fix region bundle:
  `target/fret-diag-pagination-demo-action-selected-state-v1/sessions/1779581631921-115356/1779581648658-script-step-0008-assert-failed`;
  failing pre-fix dispatch bundle:
  `target/fret-diag-pagination-demo-action-selected-state-v3/sessions/1779582115192-96036/1779582191425-script-step-0022-wait_until-timeout`;
  focused runtime AI packet after fixes:
  `target/fret-diag-pagination-demo-action-selected-state-v4/sessions/1779582812866-7968/1779582821896/ai.packet`;
  focused runtime pack after fixes:
  `target/fret-diag-pagination-demo-action-selected-state-v4/sessions/1779582812866-7968/share/1779582821896.zip`;
  dedicated suite summary:
  `target/fret-diag-pagination-action-selected-state-suite-v1/sessions/1779582840753-86300/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-demo-action-selected-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-pagination-action-selected-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --config skip_children=true --check apps/fret-ui-gallery/src/driver/runtime_driver.rs apps/fret-ui-gallery/src/ui/snippets/pagination/demo.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs ecosystem/fret-bootstrap/src/ui_diagnostics/selector.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_pagination_demo_action_selected_state --no-fail-fast --no-capture`
  (run id `6552f734-5959-432b-8cb3-8f3e19dbd9a5`);
  `cargo test --profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics semantics_region_role_label_round_trips_for_diagnostics -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib pagination -- --nocapture`
  (11/11 passed);
  `cargo test --profile dev-fast -p fret-ui-gallery driver_handled_command_dispatch_records_source_and_scope -- --nocapture`
  (1/1 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-demo-action-selected-state.json --dir target/fret-diag-pagination-demo-action-selected-state-v4 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779582821896`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-pagination-action-selected-state --dir target/fret-diag-pagination-action-selected-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779582849598`, suite run id `1779582840753-86300`, 1/1 passed).

## Tabs Demo Relation Action State

- invariant:
  the Tabs Demo must expose APG/Radix-aligned composite semantics through stable runtime selectors:
  root-derived `role=tab_list`, tab triggers with `role=tab`, `invoke=true`, selected state, and an
  active `role=tab_panel` whose `labelled_by` edge points back to the selected tab while the
  selected tab exposes the derived `controls` edge. Pointer activation and ArrowLeft automatic
  roving activation must update selected state, focus, mounted panel, and relation endpoints
  together.
- finding:
  no core relation-normalization defect was reproduced. The first focused runtime found an
  authoring/observability gap in the Gallery demo: `.test_id("ui-gallery-tabs-demo")` was stamped
  after `into_element(cx)`, so `Tabs` could not derive child selectors like
  `ui-gallery-tabs-demo-list`. The runtime script could locate the demo root and hand-authored
  trigger ids, but not the recipe-derived tablist endpoint needed for role/relation assertions.
- fix:
  `TabsContent` and direct `TabsItem` now accept content-panel `test_id`s, `Tabs` derives a
  root-scoped tablist selector, and the Gallery Tabs demo stamps its root test id at the builder
  layer before `into_element(cx)`. The demo also gives Account and Password panels stable ids so
  diagnostics can assert the active panel relation endpoint.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-demo-relation-action-state.json` starts
  directly on `tabs` / `Demo`, asserts tablist/tab/panel roles, trigger invoke actions, initial
  Account selected-state and panel relation, pointer-switches to Password and proves relation
  migration, then presses ArrowLeft and proves focus, selected state, panel mount, `controls`, and
  `labelled_by` return to Account.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/tabs.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/tabs/demo.rs`,
  `tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-demo-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-tabs-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  failing pre-fix child-id bundle:
  `target/fret-diag-tabs-demo-relation-action-state-v1/sessions/1779583914893-112252/1779583955034-script-step-0008-assert-failed`;
  focused runtime AI packet after fix:
  `target/fret-diag-tabs-demo-relation-action-state-v2/sessions/1779584075748-113716/1779584088541/ai.packet`;
  focused runtime pack after fix:
  `target/fret-diag-tabs-demo-relation-action-state-v2/sessions/1779584075748-113716/share/1779584088541.zip`;
  dedicated suite summary:
  `target/fret-diag-tabs-relation-action-state-suite-v1/sessions/1779584134695-115524/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-demo-relation-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-tabs-relation-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check ecosystem/fret-ui-shadcn/src/tabs.rs apps/fret-ui-gallery/src/ui/snippets/tabs/demo.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_tabs_demo_relation_action_state --no-fail-fast --no-capture`
  (run id `efaa1de3-c092-4126-9d89-445689d7d755`);
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib tabs_content_test_id_stamps_active_tab_panel_relation_endpoint -- --nocapture`
  (1/1 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-demo-relation-action-state.json --dir target/fret-diag-tabs-demo-relation-action-state-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779584088541`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-tabs-relation-action-state --dir target/fret-diag-tabs-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779584147936`, suite run id `1779584134695-115524`, 1/1 passed).

## Accordion Demo Relation Action State

- invariant:
  the Accordion Demo must expose Radix-aligned trigger/content semantics through stable runtime
  selectors: each trigger is a button-like invokable expanded-state source, the mounted content is
  `role=region`, the open trigger exposes `controls -> content`, and the content exposes
  `labelled_by -> trigger`. In a single collapsible accordion, pointer switching must migrate those
  relation endpoints to the newly open item, and keyboard close must clear `controls` when the
  panel unmounts.
- finding:
  no core relation-normalization, keyboard activation, or shadcn Accordion recipe defect was
  reproduced. The existing demo already had stable Shipping/Returns trigger and content ids; the
  missing piece was a promoted runtime gate tying those ids to relation/action-state assertions.
- fix:
  no runtime or recipe fix was required. Added the runtime script, suite manifest, registry entry,
  and protocol roundtrip coverage.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-demo-relation-action-state.json`
  starts directly on `accordion` / `Demo`, asserts Shipping/Returns trigger roles and invoke
  actions, opens Shipping, proves content `region` plus `controls` / `labelled_by`, switches to
  Returns and proves Shipping clears while Returns relations mount, then focuses Returns and uses
  Enter to close and prove the relation endpoint clears.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/accordion/demo.rs`,
  `ecosystem/fret-ui-shadcn/src/accordion.rs`,
  `tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-demo-relation-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-accordion-relation-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-accordion-demo-relation-action-state-v1/sessions/1779585499822-114736/1779585528462/ai.packet`;
  focused runtime pack:
  `target/fret-diag-accordion-demo-relation-action-state-v1/sessions/1779585499822-114736/share/1779585528462.zip`;
  dedicated suite summary:
  `target/fret-diag-accordion-relation-action-state-suite-v1/sessions/1779585598791-113112/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-demo-relation-action-state.json`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-accordion-relation-action-state/suite.json`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_accordion_demo_relation_action_state --no-fail-fast --no-capture`
  (run id `b88ef617-0d12-4c19-a28b-a4ddbd5ebc7f`);
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib accordion_trigger_controls_resolves_to_content_when_open -- --nocapture`
  (2/2 passed);
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib accordion_content_is_region_and_labelled_by_trigger_when_open -- --nocapture`
  (2/2 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-demo-relation-action-state.json --dir target/fret-diag-accordion-demo-relation-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  (run id `1779585528462`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-accordion-relation-action-state --dir target/fret-diag-accordion-relation-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  (script run id `1779585617613`, suite run id `1779585598791-113112`, 1/1 passed).

## ButtonGroupText Label Control Action State

- invariant:
  the ButtonGroupText docs example must preserve shadcn-style label/control semantics when a
  custom `Label::for_control` child is used as a prefix addon. The prefix label should expose a
  text role with no focus or set-value actions, publish `controls -> input`, and remain clickable
  as a label. The adjacent input should expose `role=text_field`, focus and set-value actions,
  derive `labelled_by -> prefix-label` from the control registry, receive focus when the label is
  clicked, and accept typed text after that focus transfer.
- finding:
  the first focused runtime found a Gallery authoring bug: the input had a direct
  `.a11y_label("URL")`, so `Input` intentionally skipped the registry-derived `labelled_by`
  relation from `Label::for_control`. After fixing that, the second focused runtime proved the
  semantic relation was correct but click-to-focus still timed out. The hit-test trace included
  the intended prefix label and no occlusion, while the input remained unfocused. The root cause
  was in `fret-ui-kit` primitive `Label::for_control`: the `FocusOnly` pointer-down branch
  requested focus but did not prevent the runtime's default pointer-down focus or capture the
  pointer, so ambient ancestors could keep focus instead of the registered control in wrapped-root
  compositions.
- fix:
  the ButtonGroupText example now lets the input derive its accessible label from the prefix
  `Label::for_control` and moves the contextual name to the ButtonGroup root with
  `.a11y_label("Website URL")`. `fret-ui-kit` `Label::for_control` now prevents
  `DefaultAction::FocusOnPointerDown`, captures the pointer, and stops propagation for
  `ControlAction::FocusOnly`, matching the intended label-to-control focus handoff. `FieldLabel`
  received the same FocusOnly consistency fix because it owns a parallel label-control forwarding
  path.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-text-label-control-action-state.json`
  starts directly on `button_group` / `ButtonGroupText`, asserts prefix/suffix group and label
  roles, label action suppression, input focus/set-value action exposure, label `controls`, input
  `labelled_by`, click-label focus transfer, typed value mutation to `docs`, and captures
  layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/button_group/text.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`,
  `ecosystem/fret-ui-kit/src/primitives/label.rs`,
  `ecosystem/fret-ui-shadcn/src/field.rs`,
  `ecosystem/fret-ui-shadcn/src/input.rs`,
  `tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-text-label-control-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-button-group-text-label-control-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  failing pre-fix relation bundle:
  `target/fret-diag-button-group-text-label-control-action-state-v1/sessions/1779586724469-55396/1779586758864-script-step-0018-assert-failed`;
  failing pre-fix focus bundle:
  `target/fret-diag-button-group-text-label-control-action-state-v2/sessions/1779587155678-119616/1779587326432-script-step-0024-wait_until-timeout`;
  focused runtime AI packet after fixes:
  `target/fret-diag-button-group-text-label-control-action-state-v3/sessions/1779588626684-64836/1779588722417/ai.packet`;
  focused runtime pack after fixes:
  `target/fret-diag-button-group-text-label-control-action-state-v3/sessions/1779588626684-64836/share/1779588722417.zip`;
  dedicated suite summary:
  `target/fret-diag-button-group-text-label-control-action-state-suite-v1/sessions/1779588746195-86804/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-text-label-control-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-button-group-text-label-control-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check ecosystem/fret-ui-kit/src/primitives/label.rs ecosystem/fret-ui-shadcn/src/field.rs apps/fret-ui-gallery/src/ui/snippets/button_group/text.rs apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_button_group_text_label_control_action_state --no-fail-fast --no-capture`
  (run id `d274c12f-af83-4d82-b682-3074c49ee460`);
  `cargo test --profile dev-fast -p fret-ui-kit --lib label_for_control -- --nocapture`
  (4/4 passed);
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib field_label_click -- --nocapture`
  (7/7 passed);
  `cargo test --profile dev-fast -p fret-ui-shadcn --test input_label_focus field_label_click_focuses_input_control -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-ui-shadcn --test textarea_label_focus field_label_click_focuses_textarea_control -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-ui-gallery --test ui_authoring_surface_default_app button_group_text_follow_up_teaches_label_mapping_without_slot_api -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib button_group -- --nocapture`
  (15/15 passed);
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib input_control_id -- --nocapture`
  (2/2 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-text-label-control-action-state.json --dir target/fret-diag-button-group-text-label-control-action-state-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 480000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  (run id `1779588722417`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-button-group-text-label-control-action-state --dir target/fret-diag-button-group-text-label-control-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  (script run id `1779588761135`, suite run id `1779588746195-86804`, 1/1 passed).

## Field Demo Label Control Action State

- invariant:
  the Field Demo must preserve shadcn-style label/control semantics across heterogeneous controls.
  Labels should expose text semantics and `controls` edges, concrete controls should expose the
  appropriate role/action-state plus reciprocal `labelled_by` when the label registry owns the
  accessible name, label clicks should focus text inputs and textarea controls, the checkbox label
  should toggle the checkbox, and typed diagnostics values should land in writable controls.
- finding:
  the first focused runtime drafts found authoring and diagnostics issues. The Gallery demo gave
  several concrete controls direct `.a11y_label(...)` values, so `FieldLabel::for_control(...)`
  relations were intentionally shadowed and could not be asserted as `labelled_by` edges. After
  exposing concrete control test ids and removing those direct labels, a later run failed
  `role_is card-name text_field` even though the failure bundle contained
  `ui-gallery-field-demo-card-name` with `role=text_field`, focus/set_value actions, and the
  expected label relation.
- fix:
  the Field Demo now stamps stable test ids on its label/control endpoints and lets CVV, name on
  card, card number, same-as-shipping, and comments controls derive accessible names from their
  `FieldLabel::for_control(...)` relations. The diagnostics script engine now defers
  semantics-dependent `wait_until` / `assert` predicates during no-frame keepalive ticks until a
  real frame provides current semantics, while still allowing frame-independent predicates to run.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/field/ui-gallery-field-demo-label-control-action-state.json`
  starts directly on `field` / `Demo`, asserts label roles and relation edges, control
  roles/actions, click-label focus/toggle behavior, text and textarea value mutation, and captures
  layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/field/demo.rs`,
  `apps/fret-ui-gallery/tests/field_docs_surface.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`,
  `tools/diag-scripts/ui-gallery/field/ui-gallery-field-demo-label-control-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-field-demo-label-control-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  failing pre-fix root-selector bundle:
  `target/fret-diag-field-demo-label-control-action-state-v1/sessions/1779589782434-100720/1779589864588/ai.packet`;
  failing pre-fix missing-control-id bundle:
  `target/fret-diag-field-demo-label-control-action-state-v2/sessions/1779590105841-121252/1779590114343/ai.packet`;
  failing no-frame false-assert bundle:
  `target/fret-diag-field-demo-label-control-action-state-v4/sessions/1779590766986-112780/1779590783535/ai.packet`;
  focused runtime AI packet after fixes:
  `target/fret-diag-field-demo-label-control-action-state-v7/sessions/1779592978782-55812/1779592992710/ai.packet`;
  focused runtime pack after fixes:
  `target/fret-diag-field-demo-label-control-action-state-v7/sessions/1779592978782-55812/share/1779592992710.zip`;
  dedicated suite summary:
  `target/fret-diag-field-demo-label-control-action-state-suite-v1/sessions/1779593090166-51384/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/field/ui-gallery-field-demo-label-control-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-field-demo-label-control-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/field/demo.rs apps/fret-ui-gallery/tests/field_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_field_demo_label_control_action_state --no-fail-fast --no-capture`
  (run id `1732da94-77ab-4df2-9e93-ca59ecd8506c`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test field_docs_surface field_demo_teaches_label_control_relations_without_direct_label_shadowing -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-ui-gallery --test field_docs_surface field_diag_scripts_cover_docs_smoke_and_responsive_follow_up -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-bootstrap --lib --features diagnostics,ui-app-driver no_frame_keepalive_defers_semantics_predicates_to_real_frame -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-bootstrap --lib --features diagnostics,ui-app-driver no_frame_keepalive_still_evaluates_frame_independent_predicates -- --nocapture`
  (1/1 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/field/ui-gallery-field-demo-label-control-action-state.json --dir target/fret-diag-field-demo-label-control-action-state-v7 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779592992710`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-field-demo-label-control-action-state --dir target/fret-diag-field-demo-label-control-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779593105444`, suite run id `1779593090166-51384`, 1/1 passed).

## Item Demo Link Action State

- invariant:
  the Item Demo must keep plain Item rows non-interactive unless the caller opts into an action or
  interactive render mode, while link-rendered rows must expose link semantics, focus/invoke
  actions, and route their authored app command through the diagnostics dispatch trace.
- finding:
  no mechanism or recipe defect was reproduced. The older `ui-gallery-item-link-render` smoke only
  asserted `role=link` on a separate example; this slice promotes the docs Demo row into a stronger
  runtime gate that also covers non-interactive action suppression and driver-handled command
  attribution.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/item/ui-gallery-item-demo-link-action-state.json` starts directly
  on `item` / `Demo`, asserts the Basic Item root has `focus=false` and `invoke=false`, verifies the
  link-rendered Item media/content/actions anchors, asserts `role=link`, label, focus and invoke
  actions, focuses the link row, clicks it, waits for `ui_gallery.app.open` with
  `handled_by_driver=true`, proves `/shell/last_action=cmd.open`, and captures
  layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/item/demo.rs`,
  `apps/fret-ui-gallery/tests/item_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/item/ui-gallery-item-demo-link-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-item-demo-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-item-demo-link-action-state-v1/sessions/1779594634911-118812/1779594651180/ai.packet`;
  focused runtime pack:
  `target/fret-diag-item-demo-link-action-state-v1/sessions/1779594634911-118812/share/1779594651180.zip`;
  dedicated suite summary:
  `target/fret-diag-item-demo-action-state-suite-v1/sessions/1779594685092-122432/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/item/ui-gallery-item-demo-link-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-item-demo-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/tests/item_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_item_demo_link_action_state --no-fail-fast --no-capture`
  (run id `60b589ea-6521-4160-80e9-7f28661c7b90`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test item_docs_surface -- --nocapture`
  (2/2 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/item/ui-gallery-item-demo-link-action-state.json --dir target/fret-diag-item-demo-link-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779594651180`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-item-demo-action-state --dir target/fret-diag-item-demo-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779594700725`, suite run id `1779594685092-122432`, 1/1 passed).

## Item Link-Render Action State

- invariant:
  the standalone `ItemRender::Link` example should expose link semantics and focus/invoke actions
  on the item-owned render surface, keyboard activation should be attributable to a focus-origin
  command dispatch, and pointer activation should retain the concrete source `test_id`.
- finding:
  no mechanism or recipe defect was reproduced. The previous
  `tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json` script was part of
  `ui-gallery-shadcn-conformance`, but it still navigated through search, asserted only
  `role=link`, clicked the row, and captured a bundle. The strengthened gate now turns that old
  smoke into action-state and command-dispatch evidence.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json` starts directly on
  `item` / `Link (render)`, asserts the page and link row are present, scrolls the row fully into
  view, asserts `role=link`, label `Dashboard`, focus and invoke actions, focuses the row,
  activates it via Enter, requires `ui_gallery.app.open` with `source_kind=keyboard` and
  `started_from_focus=true`, proves `/shell/last_action=cmd.open`, clicks the row, requires pointer
  dispatch with `source_test_id=ui-gallery-item-link-render`, and captures
  layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/item/link_render.rs`,
  `apps/fret-ui-gallery/tests/item_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json`,
  `tools/diag-scripts/ui-gallery-item-link-render.json`,
  `tools/diag-scripts/suites/ui-gallery-item-link-action-state/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-item-link-action-state-v1/sessions/1779612288654-129960/1779612297612/ai.packet`;
  focused runtime pack:
  `target/fret-diag-item-link-action-state-v1/sessions/1779612288654-129960/share/1779612297612.zip`;
  dedicated suite summary:
  `target/fret-diag-item-link-action-state-suite-v1/sessions/1779612316673-125332/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-item-link-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/tests/item_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `git diff --check -- tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json tools/diag-scripts/suites/ui-gallery-item-link-action-state/suite.json apps/fret-ui-gallery/tests/item_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_item_link_render script_v2_roundtrip_ui_gallery_item_demo_link_action_state --no-fail-fast --no-capture`
  (run id `d6252e0c-95cb-4318-9542-818edf50c848`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test item_docs_surface -- --nocapture`
  (4/4 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json --dir target/fret-diag-item-link-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779612297612`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-item-link-action-state --dir target/fret-diag-item-link-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779612325507`, suite run id `1779612316673-125332`, 1/1 passed).

## Empty Demo Action State

- invariant:
  the Empty Demo must keep structural Empty/Header/Title surfaces non-interactive while preserving
  action semantics on the child Buttons and link CTA. The title should remain text with no focus or
  invoke actions, ordinary buttons should export button roles plus focus/invoke actions, and
  `ButtonRender::Link` should expose link semantics with focus/invoke actions without requiring a
  generic `asChild` escape hatch.
- finding:
  no mechanism or recipe defect was reproduced. Existing Empty evidence covered docs smoke,
  layout sidecars, and screenshots; this slice closes the missing Demo-level action-state and
  focus traversal evidence.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/empty/ui-gallery-empty-demo-action-state.json` starts directly on
  `empty` / `Demo`, asserts title text semantics and action suppression, asserts Create Project and
  Import Project button semantics/actions, asserts Learn More link semantics/actions, verifies Tab
  traversal across the action row and link CTA, clicks the link CTA with the example-local
  no-op activation handler, and captures layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/empty/demo.rs`,
  `apps/fret-ui-gallery/tests/empty_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/empty/ui-gallery-empty-demo-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-empty-demo-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-empty-demo-action-state-v1/sessions/1779595537286-123920/1779595557061/ai.packet`;
  focused runtime pack:
  `target/fret-diag-empty-demo-action-state-v1/sessions/1779595537286-123920/share/1779595557061.zip`;
  dedicated suite summary:
  `target/fret-diag-empty-demo-action-state-suite-v1/sessions/1779595700919-125456/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/empty/ui-gallery-empty-demo-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-empty-demo-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/tests/empty_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_empty_demo_action_state --no-fail-fast --no-capture`
  (run id `1e7a6b23-7300-496c-84bb-cc751f071a64`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test empty_docs_surface -- --nocapture`
  (2/2 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/empty/ui-gallery-empty-demo-action-state.json --dir target/fret-diag-empty-demo-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779595557061`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-empty-demo-action-state --dir target/fret-diag-empty-demo-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779595720079`, suite run id `1779595700919-125456`, 1/1 passed).

## Card Demo Action State

- invariant:
  the Card Demo login composition must keep title/description text structural while preserving
  action semantics on form fields, the supporting link, and Card header/footer actions. Text fields
  should expose focus/set-value actions, link chrome should expose link semantics with
  focus/invoke actions, and link-variant/ordinary buttons should remain button action endpoints.
- finding:
  no mechanism or recipe defect was reproduced. Existing Card evidence covered docs-path smoke,
  screenshots, and retained-memory analysis; this slice closes the missing Demo-level action-state
  and text-input mutation evidence.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/card/ui-gallery-card-demo-action-state.json` starts directly on
  `card` / `Demo`, asserts structural title/description text semantics, asserts email/password
  text-field roles and actions, asserts the Forgot Password chrome link semantics/actions, asserts
  Sign Up/Login/Login with Google button semantics/actions, types `ada@example.com` into the email
  field, proves the value mutation, focuses the password field, clicks Login, and captures
  layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/card/demo.rs`,
  `apps/fret-ui-gallery/tests/card_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/card/ui-gallery-card-demo-action-state.json`,
  `tools/diag-scripts/suites/ui-gallery-card-demo-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-card-demo-action-state-v1/sessions/1779596562467-100040/1779596574143/ai.packet`;
  focused runtime pack:
  `target/fret-diag-card-demo-action-state-v1/sessions/1779596562467-100040/share/1779596574143.zip`;
  dedicated suite summary:
  `target/fret-diag-card-demo-action-state-suite-v1/sessions/1779596642442-120232/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-demo-action-state.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-card-demo-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/tests/card_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_card_demo_action_state --no-fail-fast --no-capture`
  (run id `1562d937-a6ec-4517-a116-e31921cdfaad`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test card_docs_surface card_demo_action_state_gate_keeps_runtime_anchors -- --nocapture`
  (1/1 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/card/ui-gallery-card-demo-action-state.json --dir target/fret-diag-card-demo-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779596574143`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-card-demo-action-state --dir target/fret-diag-card-demo-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779596653764`, suite run id `1779596642442-120232`, 1/1 passed).

## Badge Link-Render Action State

- invariant:
  `BadgeRender::Link` should expose link semantics and focus/invoke actions on the badge-owned
  render surface, while the surrounding docs row stays a layout concern and must not be required to
  carry interaction semantics.
- finding:
  no mechanism or recipe defect was reproduced. The first focused draft failed because the script
  over-asserted `ui-gallery-badge-link-row` as `role=group`; runtime slice evidence showed the row
  is a `generic` layout container, while `ui-gallery-badge-link` correctly exports `role=link`,
  label `Open Link`, and focus/invoke actions. The final gate narrows the row check to existence
  and keeps the semantic action contract on the Badge link node itself.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json` now starts directly on
  `badge` / `Link`, asserts the row and derived `.chrome` marker exist, asserts link role, label,
  focus and invoke actions, focuses the link, activates it with Enter, clicks the no-op example
  handler, and captures layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/badge/link.rs`,
  `apps/fret-ui-gallery/tests/badge_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json`,
  `tools/diag-scripts/ui-gallery-badge-link-render.json`,
  `tools/diag-scripts/suites/ui-gallery-badge-link-action-state/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  over-specific pre-oracle bundle:
  `target/fret-diag-badge-link-action-state-v1/sessions/1779597372964-123748/1779597405221-script-step-0008-assert-failed`;
  focused runtime AI packet:
  `target/fret-diag-badge-link-action-state-v2/sessions/1779597589387-126604/1779597602835/ai.packet`;
  focused runtime pack:
  `target/fret-diag-badge-link-action-state-v2/sessions/1779597589387-126604/share/1779597602835.zip`;
  dedicated suite summary:
  `target/fret-diag-badge-link-action-state-suite-v1/sessions/1779597642170-110112/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-badge-link-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/tests/badge_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_badge_link_render --no-fail-fast --no-capture`
  (post-oracle run id `96fe8e8d-d742-4c27-95cf-1cce907755c5`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test badge_docs_surface -- --nocapture`
  (2/2 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json --dir target/fret-diag-badge-link-action-state-v2 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779597602835`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-badge-link-action-state --dir target/fret-diag-badge-link-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779597652401`, suite run id `1779597642170-110112`, 1/1 passed).

## Button Link-Render Action State

- invariant:
  `ButtonRender::Link` should expose link semantics and focus/invoke actions on the button-owned
  render surface, keyboard activation should be attributable to focus-origin command dispatch, and
  pointer activation should keep the concrete source `test_id`. Surrounding docs row/chrome nodes
  are anchors for observability, not the owners of the link role.
- finding:
  the first strengthened script used the wrong DocSection prefix and timed out waiting for
  `ui-gallery-button-link-semantic`; runtime slice evidence showed the actual stable content id is
  `ui-gallery-button-link-semantic-content`. After fixing the script oracle, the next focused run
  found a real diagnostics attribution defect: the keyboard Enter activation produced a pending
  keyboard source with `started_from_focus=true`, but the later driver-handled trace for
  `ui_gallery.app.open` recorded `started_from_focus=false`. Pointer dispatch attribution was
  already correct.
- fix:
  the canonical Button link-render script now starts directly on `button` /
  `As Link / As Child (Semantic)` and asserts section content, row/chrome anchors, link role,
  label, actions, keyboard dispatch, app snapshot mutation, pointer dispatch, and bundle captures.
  Driver-handled command dispatch tracing now preserves focus-origin keyboard activation in the UI
  Gallery driver, the default bootstrap app driver, and Workspace Shell demo driver. Keyboard
  pending sources are classified as `started_from_focus=true`; shortcuts and pointer sources remain
  separate source kinds.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json` asserts
  `role=link`, label `Login`, `focus=true`, `invoke=true`, focused Enter activation,
  `ui_gallery.app.open` handled by the driver with `source_kind=keyboard` and
  `started_from_focus=true`, `/shell/last_action=cmd.open`, pointer click dispatch with
  `source_test_id=ui-gallery-button-render-link`, and captures layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/button/link_render.rs`,
  `apps/fret-ui-gallery/tests/button_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json`,
  `tools/diag-scripts/ui-gallery-button-link-render.json`,
  `tools/diag-scripts/suites/ui-gallery-button-link-action-state/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`,
  `apps/fret-ui-gallery/src/driver/runtime_driver.rs`,
  `ecosystem/fret-bootstrap/src/ui_app_driver.rs`,
  `apps/fret-examples/src/workspace_shell_demo.rs`.
- evidence anchors:
  wrong-section-id failure bundle:
  `target/fret-diag-button-link-action-state-v1/sessions/1779598532428-94836/1779598752440-script-step-0005-wait_until-timeout`;
  pre-fix keyboard trace failure AI packet:
  `target/fret-diag-button-link-action-state-v2/sessions/1779598995024-118036/1779599006627/ai.packet`;
  pre-fix keyboard trace pack:
  `target/fret-diag-button-link-action-state-v2/sessions/1779598995024-118036/share/1779599006627.zip`;
  pre-fix keyboard trace timeout bundle:
  `target/fret-diag-button-link-action-state-v2/sessions/1779598995024-118036/1779599060978-script-step-0017-wait_command_dispatch_trace-timeout`;
  focused runtime AI packet after fixes:
  `target/fret-diag-button-link-action-state-v3/sessions/1779599727232-124860/1779599736183/ai.packet`;
  focused runtime pack after fixes:
  `target/fret-diag-button-link-action-state-v3/sessions/1779599727232-124860/share/1779599736183.zip`;
  dedicated suite summary:
  `target/fret-diag-button-link-action-state-suite-v1/sessions/1779599758073-43676/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-button-link-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check --config skip_children=true apps/fret-ui-gallery/tests/button_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs apps/fret-ui-gallery/src/driver/runtime_driver.rs ecosystem/fret-bootstrap/src/ui_app_driver.rs apps/fret-examples/src/workspace_shell_demo.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_button_link_render --no-fail-fast --no-capture`
  (run id `41dcf93c-4bc4-49c3-93a8-1bfba17c4883`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test button_docs_surface -- --nocapture`
  (2/2 passed);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui mechanism_harness_pressable_key_activation_matches_oracles --no-fail-fast --no-capture`
  (run id `41da1eca-4ea9-425c-825a-8a7f2f3bb4a5`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  `cargo build --profile dev-fast -p fret-examples`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json --dir target/fret-diag-button-link-action-state-v3 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779599736183`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-button-link-action-state --dir target/fret-diag-button-link-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779599767438`, suite run id `1779599758073-43676`, 1/1 passed).

## Typography Inline Link Action State

- invariant:
  `p_rich(...).inline_link(...)` should remain a selectable-text mechanism surface: the paragraph
  owns `role=text`, exposes `set_text_selection`, publishes inline span role/tag metadata, and
  activation is component policy through `on_activate_link(...)`, not a separate button/link node.
- finding:
  no Typography or selectable-text runtime defect was reproduced. The slice did find a diagnostics
  observability gap: scripts could activate a selectable-text span by tag but could not directly
  assert that the semantics snapshot contained the inline span role/tag metadata that made the
  click meaningful.
- fix:
  the diagnostics protocol now includes `semantics_inline_span_includes`, with a typed builder
  helper, bootstrap predicate evaluator support, selector-resolution tracing, and focused protocol
  plus executor tests. The canonical Typography Interactive Links script now starts directly on
  `typography` / `Interactive Links`, asserts paragraph text/value/action/span metadata, activates
  the `https://example.com/kings-plan` span, and verifies the app-owned status switches from idle
  to active.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json`
  asserts `role=text`, `value_contains=a brilliant plan`, `set_text_selection=true`,
  `semantics_inline_span_includes(role=link, tag=https://example.com/kings-plan)`,
  `click_selectable_text_span_stable`, and active/idle status mutation, then captures
  layout/screenshot/bundle evidence.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/pages/typography.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/typography/interactive_links.rs`,
  `apps/fret-ui-gallery/tests/typography_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json`,
  `tools/diag-scripts/ui-gallery-typography-interactive-links-activation.json`,
  `tools/diag-scripts/suites/ui-gallery-typography-inline-link-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/src/lib.rs`,
  `crates/fret-diag-protocol/src/builder.rs`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-typography-inline-link-action-state-v1/sessions/1779603310309-127364/1779603327158/ai.packet`;
  focused runtime pack:
  `target/fret-diag-typography-inline-link-action-state-v1/sessions/1779603310309-127364/share/1779603327158.zip`;
  dedicated suite summary:
  `target/fret-diag-typography-inline-link-action-state-suite-v1/sessions/1779603371227-113256/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-typography-inline-link-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates/fret-diag-protocol/src/lib.rs crates/fret-diag-protocol/src/builder.rs ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs apps/fret-ui-gallery/tests/typography_docs_surface.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol predicate_semantics_inline_span_includes_serializes_and_deserializes --no-fail-fast --no-capture`
  (run id `aa8c2b87-1d29-4888-9028-0cae8927c832`);
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_typography_interactive_links_activation --no-fail-fast --no-capture`
  (run id `19d9cab9-2da6-4570-ac16-627494a9e66f`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test typography_docs_surface -- --nocapture`
  (3/3 passed);
  `cargo nextest run --cargo-profile dev-fast -p fret-bootstrap --features ui-app-driver,diagnostics semantics_inline_span_predicates_match_inline_link_metadata --no-fail-fast --no-capture`
  (run id `0780b162-d707-4b5b-9dc6-46e74cb5e75d`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json --dir target/fret-diag-typography-inline-link-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779603327158`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-typography-inline-link-action-state --dir target/fret-diag-typography-inline-link-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779603386622`, suite run id `1779603371227-113256`, 1/1 passed).

## Alert Link Action State

- invariant:
  AlertDescription-composed pressable links should expose semantic link role, accessible label, URL
  value, focus action, and invoke action on the concrete link nodes. Keyboard Enter activation and
  pointer activation should update the app-owned diagnostics status without opening external URLs in
  diagnostics mode.
- finding:
  no Alert recipe or pressable runtime defect was reproduced. The existing script was an old
  navigation smoke that clicked only the Billing link and waited for a status marker; it did not
  prove direct-start section entry, Support-link metadata, URL values, focus action exposure, or
  keyboard activation.
- fix:
  the canonical Alert link script now starts directly on `alert` / `Interactive Links`, asserts the
  page and section anchors, proves both Billing and Support link role/label/value/action metadata,
  activates Billing from keyboard focus, clicks Support with a stable pointer click, and captures
  layout/screenshot/bundle evidence. A dedicated `ui-gallery-alert-link-action-state` suite and
  Gallery authoring test now lock the strengthened gate while preserving the legacy redirect stub
  and shadcn-conformance membership.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-link-activation.json` asserts
  `role=link`, `label_contains`, `value_contains`, `semantics_action_is(focus=true)`,
  `semantics_action_is(invoke=true)`, `focus_is`, keyboard Enter activation, pointer click
  activation, and status mutation for both Alert links.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/pages/alert.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/alert/interactive_links.rs`,
  `apps/fret-ui-gallery/tests/alert_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-link-activation.json`,
  `tools/diag-scripts/ui-gallery-alert-link-activation.json`,
  `tools/diag-scripts/suites/ui-gallery-alert-link-action-state/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-alert-link-action-state-v1/sessions/1779604692279-129204/1779604711175/ai.packet`;
  focused runtime pack:
  `target/fret-diag-alert-link-action-state-v1/sessions/1779604692279-129204/share/1779604711175.zip`;
  dedicated suite summary:
  `target/fret-diag-alert-link-action-state-suite-v1/sessions/1779604866072-98788/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-link-activation.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-alert-link-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/tests/alert_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_alert_link_activation --no-fail-fast --no-capture`
  (run id `1bf2754a-c4ca-4738-b39b-e763dcb09b2d`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test alert_docs_surface alert_interactive_links_diag_script_gates_action_state -- --nocapture`
  (1/1 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-link-activation.json --dir target/fret-diag-alert-link-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779604711175`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-alert-link-action-state --dir target/fret-diag-alert-link-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779604885723`, suite run id `1779604866072-98788`, 1/1 passed).

## Markdown Span Link Action State

- invariant:
  Markdown selectable-text links should expose text role, full value text, set-text-selection
  action support, and inline link span metadata on the selectable-text node. Activation should
  update the app-owned readout without relying on navigation search or a generic smoke path.
- finding:
  no Markdown renderer or selectable-text runtime defect was reproduced. The old script still used
  navigation search and only proved activation, so the span-link metadata and direct-start path were
  not yet locked.
- fix:
  the canonical Markdown span-link script now starts directly on the `markdown_editor_source` dev
  page, asserts the page root, editor root, span gate, `role=text`, `value_contains`, selection
  action exposure, inline span metadata, and activated readout, and captures layout/screenshot/
  bundle evidence. A dedicated `ui-gallery-markdown-span-link-action-state` suite and Gallery
  authoring test now lock the span-link gate while preserving the legacy redirect stub and existing
  text-wrap suite continuity.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-markdown-span-link-gate-activate.json`
  asserts `role=text`, `value_contains=https://example.com`,
  `semantics_action_is(set_text_selection=true)`,
  `semantics_inline_span_includes(role=link, tag=https://example.com)`, stable span activation,
  and the activated readout.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/markdown.rs`,
  `apps/fret-ui-gallery/tests/markdown_editor_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-markdown-span-link-gate-activate.json`,
  `tools/diag-scripts/ui-gallery-markdown-span-link-gate-activate.json`,
  `tools/diag-scripts/suites/ui-gallery-markdown-span-link-action-state/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-text-wrap/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/predicates.rs`,
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`.
- evidence anchors:
  focused runtime AI packet:
  `target/fret-diag-markdown-span-link-action-state-v1/sessions/1779606095997-126196/1779606113404/ai.packet`;
  focused runtime pack:
  `target/fret-diag-markdown-span-link-action-state-v1/sessions/1779606095997-126196/share/1779606113404.zip`;
  dedicated suite summary:
  `target/fret-diag-markdown-span-link-action-state-suite-v1/sessions/1779606213201-114820/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-markdown-span-link-gate-activate.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-markdown-span-link-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/tests/markdown_editor_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_markdown_span_link_gate_activate --no-fail-fast --no-capture`
  (run id `375a7d6e-4ae4-4b82-b1af-eb1e5a1c762d`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test markdown_editor_docs_surface markdown_span_link_diag_script_gates_action_state -- --nocapture`
  (1/1 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-markdown-span-link-gate-activate.json --dir target/fret-diag-markdown-span-link-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779606113404`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-markdown-span-link-action-state --dir target/fret-diag-markdown-span-link-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779606229614`, suite run id `1779606213201-114820`, 1/1 passed).

## AlertDialog Destructive Inline Link Action State

- invariant:
  AlertDialog Destructive description text should remain a selectable-text mechanism surface inside
  modal content: the dialog content owns `role=alert_dialog`, the description owns `role=text`,
  exposes `set_text_selection`, publishes inline span role/tag metadata for Settings, and stable
  span activation must keep working after screenshots and cache-hit frames.
- finding:
  two issues were reproduced. The first was an authoring mismatch: diagnostics role strings use
  `alert_dialog`, not `alertdialog`. The second was a real mechanism defect: paint-cache replay
  skipped selectable-text paint and did not keep `SelectableTextState.interactive_span_bounds`
  live in the current runtime state buffer, so `click_selectable_text_span_stable` eventually timed
  out with `no_runtime_state` even though semantics, hit testing, and inline span metadata were
  still present.
- fix:
  paint-cache replay now touches selectable-text state for every element in the replayed subtree,
  preserving previously computed interactive span bounds across repeated cache-hit frames. The
  AlertDialog script now starts directly on `alert_dialog` / `Destructive`, asserts the modal
  content role, description text/action/span metadata, captures a layout sidecar and screenshots,
  and activates the Settings span through the stable selectable-text span click helper.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-destructive-inline-link-activate.json`
  asserts `role=alert_dialog`, description `role=text`, `value_contains=View Settings`,
  `semantics_action_is(set_text_selection=true)`,
  `semantics_inline_span_includes(role=link, tag=https://example.com/settings)`,
  `capture_layout_sidecar`, screenshots, stable span activation, and a final bundle.
- implementation anchors:
  `crates/fret-ui/src/elements/runtime.rs`,
  `crates/fret-ui/src/tree/paint/node.rs`,
  `crates/fret-ui/src/tree/tests/paint_cache.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/destructive.rs`,
  `apps/fret-ui-gallery/tests/alert_dialog_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-destructive-inline-link-activate.json`,
  `tools/diag-scripts/suites/ui-gallery-alert-dialog-inline-link-action-state/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  pre-fix role-string failure bundle:
  `target/fret-diag-alert-dialog-inline-link-action-state-v1/sessions/1779607109957-124104/1779607143293-script-step-0009-assert-failed`;
  pre-fix runtime-state failure bundle:
  `target/fret-diag-alert-dialog-inline-link-action-state-v2/sessions/1779607651727-126360/1779607807424-script-step-0017-click_selectable_span-timeout`;
  focused runtime AI packet:
  `target/fret-diag-alert-dialog-inline-link-action-state-v4/sessions/1779609853670-94944/1779609862553/ai.packet`;
  focused runtime pack:
  `target/fret-diag-alert-dialog-inline-link-action-state-v4/sessions/1779609853670-94944/share/1779609862553.zip`;
  dedicated suite summary:
  `target/fret-diag-alert-dialog-inline-link-action-state-suite-v2/sessions/1779609886205-109624/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-destructive-inline-link-activate.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-alert-dialog-inline-link-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check crates/fret-ui/src/elements/runtime.rs crates/fret-ui/src/tree/paint/node.rs crates/fret-ui/src/tree/tests/paint_cache.rs apps/fret-ui-gallery/tests/alert_dialog_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-ui paint_cache_replay_touches_selectable_text_span_state_for_replayed_subtrees --no-fail-fast --no-capture`
  (run id `0c2d679c-4164-49ae-8d60-e737f391ca9b`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui selectable_text_records_interactive_span_bounds_after_paint --no-fail-fast --no-capture`
  (run id `8cb793c5-26e1-4cbe-8257-96065f60de30`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui paint_cache_replay_translates_descendant_bounds_for_descendants --no-fail-fast --no-capture`
  (run id `90cb2ec0-2fdc-4a6c-81cc-2bb9dbb263c5`);
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_alert_dialog_destructive_inline_link_activate --no-fail-fast --no-capture`
  (run id `be1ab63d-ceef-4d08-9429-b5088789a071`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test alert_dialog_docs_surface alert_dialog_docs_diag_scripts_cover_docs_path_and_existing_regression_gates -- --nocapture`
  (1/1 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-destructive-inline-link-activate.json --dir target/fret-diag-alert-dialog-inline-link-action-state-v4 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779609862553`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-alert-dialog-inline-link-action-state --dir target/fret-diag-alert-dialog-inline-link-action-state-suite-v2 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779609894817`, suite run id `1779609886205-109624`, 1/1 passed).

## Breadcrumb Usage/Dropdown Link Action State

- invariant:
  ordinary `BreadcrumbLink` nodes on both the curated Usage lane and raw-primitives Dropdown lane
  should expose semantic link role, accessible label, URL value, focus action, and invoke action
  while app-bound links dispatch through the command pipeline instead of opening an external
  browser during diagnostics.
- finding:
  no Breadcrumb recipe or pressable runtime defect was reproduced. The older
  `ui-gallery-breadcrumb-usage-home-command.json` script still used navigation search and only
  asserted `role=link` plus `cmd.open`, so it did not prove direct-start section entry, URL value
  publication, action exposure, keyboard activation attribution, or pointer source attribution.
  The conformance-held `ui-gallery-breadcrumb-links-semantic-link.json` script had the same
  navigation-search and role-only limitation on the raw-primitives Dropdown Home link, and the
  snippet used a no-op activation handler that left click behavior unobservable.
- fix:
  the script now starts directly on `breadcrumb` / `Usage`, asserts Home and Components link
  metadata, focuses Home and activates it with Enter, requires a driver-handled keyboard command
  dispatch with `started_from_focus=true`, clicks Components with a stable pointer click, and
  captures layout/screenshot/bundle evidence. The Dropdown script now starts directly on
  `breadcrumb` / `Dropdown`, asserts the raw-primitives Home link metadata, verifies the adjacent
  dropdown trigger remains a separate `role=button`, and exercises keyboard plus pointer command
  dispatch. The Dropdown snippet now uses the same deterministic `ui_gallery.app.open` command
  action as Usage. A dedicated `ui-gallery-breadcrumb-link-action-state` suite and Gallery
  authoring test now lock both gates.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-usage-home-command.json` asserts
  `role=region`, `role=link`, `label_contains`, `value_contains`,
  `semantics_action_is(focus=true)`, `semantics_action_is(invoke=true)`, `focus_is`,
  `wait_command_dispatch_trace`, `/shell/last_action=cmd.open`, layout sidecar, screenshots, and a
  final bundle. `tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-links-semantic-link.json`
  applies the same action-state contract to the Dropdown Home link.
- implementation anchors:
  `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/usage.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/dropdown.rs`,
  `apps/fret-ui-gallery/tests/breadcrumb_docs_surface.rs`,
  `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`,
  `tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-usage-home-command.json`,
  `tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-links-semantic-link.json`,
  `tools/diag-scripts/ui-gallery-breadcrumb-links-semantic-link.json`,
  `tools/diag-scripts/ui-gallery/misc/ui-gallery-breadcrumb-links-semantic-link.json`,
  `tools/diag-scripts/suites/ui-gallery-breadcrumb-link-action-state/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  focused Usage runtime AI packet:
  `target/fret-diag-breadcrumb-link-action-state-v1/sessions/1779610651964-3356/1779610661121/ai.packet`;
  focused Usage runtime pack:
  `target/fret-diag-breadcrumb-link-action-state-v1/sessions/1779610651964-3356/share/1779610661121.zip`;
  focused Dropdown runtime AI packet:
  `target/fret-diag-breadcrumb-dropdown-link-action-state-v1/sessions/1779611299035-128284/1779611307893/ai.packet`;
  focused Dropdown runtime pack:
  `target/fret-diag-breadcrumb-dropdown-link-action-state-v1/sessions/1779611299035-128284/share/1779611307893.zip`;
  dedicated suite summary:
  `target/fret-diag-breadcrumb-link-action-state-suite-v2/sessions/1779611330691-124372/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-usage-home-command.json > $null`;
  `python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-links-semantic-link.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-breadcrumb-link-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `rustfmt --edition 2024 --check apps/fret-ui-gallery/src/ui/snippets/breadcrumb/dropdown.rs apps/fret-ui-gallery/tests/breadcrumb_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_breadcrumb_links_semantic_link script_v2_roundtrip_ui_gallery_breadcrumb_usage_home_command --no-fail-fast --no-capture`
  (run id `3c534f2a-e6c5-4570-b785-8b2e865e02df`);
  `cargo test --profile dev-fast -p fret-ui-gallery --test breadcrumb_docs_surface -- --nocapture`
  (4/4 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-usage-home-command.json --dir target/fret-diag-breadcrumb-link-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779610661121`);
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-links-semantic-link.json --dir target/fret-diag-breadcrumb-dropdown-link-action-state-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779611307893`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-breadcrumb-link-action-state --dir target/fret-diag-breadcrumb-link-action-state-suite-v2 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run ids `1779611339499`, `1779611360379`, suite run id `1779611330691-124372`, 2/2 passed).

## NavigationMenu Docs Smoke Action State

- invariant:
  the canonical NavigationMenu docs smoke should prove the real docs-page command and action-state
  contract, not only open/close presence. The docs-demo and RTL snippets must keep deterministic
  `ui_gallery.app.open` action anchors; closed and open triggers must expose action state; the
  expanded viewport content link, not the top-level contentless Docs link, is the correct
  roving-focus command target; and keyboard/pointer activation must leave command-dispatch trace
  evidence.
- finding:
  v1 failed on an early wait, v2 failed on a closed-state assertion, and v3/v4 timed out waiting
  for command-trace evidence. The root cause was in the shadcn recipe layer rather than `fret-ui`:
  normal `NavigationMenuLink` activation did not use the shared pressable command dispatch path,
  and the contentless top-level item replaced its command hook when installing close behavior.
- fix:
  `NavigationMenuLink` now dispatches through `pressable_dispatch_command_if_enabled_opt`, and the
  contentless top-level item appends its close handler instead of overwriting the command hook.
  The canonical docs smoke script now gates the real docs-page action-state and command-dispatch
  surface, and the dedicated `ui-gallery-navigation-menu-docs-smoke-action-state` suite promotes
  that script without replacing its shadcn-conformance or shadcn-runtime-evidence memberships.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-smoke.json`
  starts directly on `navigation_menu`, asserts docs-demo and RTL action anchors, proves closed/open
  action state, targets the expanded content link for command attribution, waits for command traces,
  captures layout/screenshots/bundle evidence, and remains the canonical NavigationMenu docs-page
  proof surface.
- implementation anchors:
  `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/rtl.rs`,
  `apps/fret-ui-gallery/tests/navigation_menu_docs_surface.rs`,
  `tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-smoke.json`,
  `tools/diag-scripts/suites/ui-gallery-navigation-menu-docs-smoke-action-state/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json`,
  `tools/diag-scripts/index.json`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`.
- evidence anchors:
  v1 early-wait failure packet:
  `target/fret-diag-navigation-menu-docs-smoke-action-state-v1/sessions/1779612961611-89860/1779612971191/ai.packet`;
  v2 closed-state failure packet:
  `target/fret-diag-navigation-menu-docs-smoke-action-state-v2/sessions/1779613175695-128644/1779613185620/ai.packet`;
  v3 command-trace timeout packet:
  `target/fret-diag-navigation-menu-docs-smoke-action-state-v3/sessions/1779613730683-124312/1779613740207/ai.packet`;
  v4 command-trace timeout packet:
  `target/fret-diag-navigation-menu-docs-smoke-action-state-v4/sessions/1779614478852-133068/1779614491391/ai.packet`;
  focused runtime AI packet after the fix:
  `target/fret-diag-navigation-menu-docs-smoke-action-state-v5/sessions/1779614850511-114228/1779614863615/ai.packet`;
  focused runtime pack after the fix:
  `target/fret-diag-navigation-menu-docs-smoke-action-state-v5/sessions/1779614850511-114228/share/1779614863615.zip`;
  dedicated suite summary:
  `target/fret-diag-navigation-menu-docs-smoke-action-state-suite-v1/sessions/1779614903581-127224/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-smoke.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-navigation-menu-docs-smoke-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_navigation_menu_docs_smoke -- --nocapture`;
  `cargo test --profile dev-fast -p fret-ui-gallery --test navigation_menu_docs_surface navigation_menu_docs_smoke_gates_demo_and_rtl_action_state -- --nocapture`;
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib navigation_menu -- --nocapture`
  (31/31 passed);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  focused runtime `target/dev-fast/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-smoke.json --dir target/fret-diag-navigation-menu-docs-smoke-action-state-v5 --session-auto --pack --ai-packet --include-triage --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779614863615`);
  dedicated suite `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-navigation-menu-docs-smoke-action-state --dir target/fret-diag-navigation-menu-docs-smoke-action-state-suite-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (script run id `1779614919851`, suite run id `1779614903581-127224`, 1/1 passed).
- fresh verification on 2026-05-24:
  `python -m json.tool tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-smoke.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-navigation-menu-docs-smoke-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py`;
  `git diff --check -- docs/workstreams/fret-mechanism-harness-v1/TODO.md docs/workstreams/fret-mechanism-harness-v1/EVIDENCE_AND_GATES.md docs/workstreams/fret-mechanism-harness-v1/COVERAGE_MAP.md ecosystem/fret-ui-shadcn/src/navigation_menu.rs tools/diag-scripts/ui-gallery/navigation-menu/ui-gallery-navigation-menu-docs-smoke.json apps/fret-ui-gallery/tests/navigation_menu_docs_surface.rs crates/fret-diag-protocol/tests/script_json_roundtrip.rs`;
  `cargo test --profile dev-fast -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_ui_gallery_navigation_menu_docs_smoke -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-ui-gallery --test navigation_menu_docs_surface navigation_menu_docs_smoke_gates_demo_and_rtl_action_state -- --nocapture`
  (1/1 passed);
  `cargo test --profile dev-fast -p fret-ui-shadcn --lib navigation_menu -- --nocapture`
  (31/31 passed; emitted an existing non-blocking `fret-ui` dead-code warning for
  `current_effective_opacity`).

## Menubar Keyboard Nav / Escape Focus Restore

- invariant:
  the canonical Menubar keyboard-nav docs path should prove that File opens the menu, ArrowDown
  lands on the app-owned command item, and Escape returns focus to the trigger.
- finding:
  no Menubar recipe defect was reproduced. The only verification drift was a stale docs-surface
  assertion about the usage snippet import order; the live snippet currently imports
  `AppComponentCx` before `UiChild`, so the guard was refreshed to match the current source.
- diagnostics surface:
  `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-keyboard-nav.json` remains the
  canonical script. `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-escape-exits-active.json`
  stays as the redirect alias, and the dedicated
  `tools/diag-scripts/suites/ui-gallery-menubar-keyboard-nav-action-state/suite.json` suite now
  promotes the canonical script.
- implementation anchors:
  `apps/fret-ui-gallery/tests/menubar_docs_surface.rs`,
  `crates/fret-diag-protocol/tests/script_json_roundtrip.rs`,
  `tools/diag-scripts/suites/ui-gallery-menubar-keyboard-nav-action-state/suite.json`,
  `tools/diag-scripts/index.json`.
- evidence anchors:
  focused runtime suite summary:
  `target/fret-diag-menubar-keyboard-nav-action-state-v1/sessions/1779619620521-132232/suite.summary.json`.
- run results:
  `python -m json.tool tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-keyboard-nav.json > $null`;
  `python -m json.tool tools/diag-scripts/suites/ui-gallery-menubar-keyboard-nav-action-state/suite.json > $null`;
  `python tools/check_diag_scripts_registry.py --write`;
  `python tools/check_diag_scripts_registry.py`;
  `cargo nextest run --cargo-profile dev-fast -p fret-diag-protocol script_v2_roundtrip_ui_gallery_menubar_keyboard_nav script_v2_roundtrip_ui_gallery_menubar_escape_exits_active --no-fail-fast --no-capture`
  (run id `23fe6fe6-f8aa-44da-a19a-dcc4274c5283`);
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-gallery --test menubar_docs_surface --no-fail-fast --no-capture`
  (run id `ef8805f0-524a-4620-97bf-38ab73e89876`);
  `cargo build --profile dev-fast -p fretboard-dev -p fret-ui-gallery --features gallery-dev`;
  `target/dev-fast/fretboard-dev.exe diag suite ui-gallery-menubar-keyboard-nav-action-state --dir target/fret-diag-menubar-keyboard-nav-action-state-v1 --session-auto --timeout-ms 600000 --launch -- target/dev-fast/fret-ui-gallery.exe`
  (run id `1779619632698`).
- fresh verification on 2026-05-24:
  the registry refresh passed after `--write`, the gallery docs-surface guard passed after the
  import-order fix, the protocol roundtrip tests passed, the build passed, and the runtime suite
  passed 1/1 with summary
  `target/fret-diag-menubar-keyboard-nav-action-state-v1/sessions/1779619620521-132232/suite.summary.json`.
