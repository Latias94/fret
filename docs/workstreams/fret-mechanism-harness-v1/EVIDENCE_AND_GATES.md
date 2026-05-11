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
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_roving_focus_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui --lib mechanism_harness_focus_scope_interaction_matches_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness_recipe_layout_cases_match_oracles -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test focus_restore_mechanism_harness mechanism_harness_focus_restore_recipe_cases_match_oracles -- --nocapture
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
cargo test --profile dev-fast -p fret-ui-shadcn --test dialog_overlay_click_dismiss_focus_restore dialog_overlay_click_closes_and_restores_focus_to_trigger -- --nocapture
cargo test --profile dev-fast -p fret-ui-shadcn --test popover_outside_click_dismiss_focus_restore popover_outside_click_closes_and_activates_underlay -- --nocapture
```

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

## Repo Integrity Gates

```powershell
python -m json.tool docs/workstreams/fret-mechanism-harness-v1/WORKSTREAM.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/layout_dirty_invalidation_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/scroll_handle_invalidation_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/environment_view_cache_invalidation_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/semantics_relations_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/roving_focus_interaction_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/declarative/tests/fixtures/focus_scope_interaction_v1.json | Out-Null
python -m json.tool ecosystem/fret-ui-shadcn/tests/fixtures/focus_restore_recipe_cases_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/pointer_occlusion_routing_v1.json | Out-Null
python -m json.tool crates/fret-ui/src/tree/tests/fixtures/focus_barrier_routing_v1.json | Out-Null
python tools/check_workstream_catalog.py
python tools/check_diag_scripts_registry.py
cargo fmt -p fret-mechanism-harness -p fret-ui --check
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
- Roving focus interaction fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/roving_focus_interaction_v1.json`
- Roving focus interaction runner:
  `crates/fret-ui/src/declarative/tests/roving_focus_interaction_harness.rs`
- Focus scope interaction fixture:
  `crates/fret-ui/src/declarative/tests/fixtures/focus_scope_interaction_v1.json`
- Focus scope interaction runner:
  `crates/fret-ui/src/declarative/tests/focus_scope_interaction_harness.rs`
- Shadcn focus restore recipe fixture:
  `ecosystem/fret-ui-shadcn/tests/fixtures/focus_restore_recipe_cases_v1.json`
- Shadcn focus restore recipe runner:
  `ecosystem/fret-ui-shadcn/tests/focus_restore_mechanism_harness.rs`
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
  `ecosystem/fret-ui-shadcn/tests/dialog_overlay_click_dismiss_focus_restore.rs`,
  `ecosystem/fret-ui-shadcn/tests/popover_outside_click_dismiss_focus_restore.rs`
- Retained virtual-list focused test: `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`
- Scroll registry classification tests: `crates/fret-ui/src/declarative/frame.rs`
- Scroll-contained frontier focused test: `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
- Layout request attribution focused test:
  `crates/fret-ui/src/tree/tests/interactive_resize_flow_rebuild.rs`
- Runtime script:
  `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-demo-with-title-toggle-underflow.json`
