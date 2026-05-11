---
title: Fret Mechanism Harness v1 Evidence and Gates
status: active
date: 2026-05-11
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
cargo test --profile dev-fast -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness_recipe_layout_cases_match_oracles -- --nocapture
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
- Previous focused tests: `crates/fret-ui/src/tree/tests/subtree_layout_dirty_underflow_repair.rs`
- View-cache focused tests: `crates/fret-ui/src/tree/tests/view_cache.rs`
- Environment focused tests: `crates/fret-ui/src/declarative/tests/environment_queries.rs`
- Retained virtual-list focused test: `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`
- Scroll registry classification tests: `crates/fret-ui/src/declarative/frame.rs`
- Scroll-contained frontier focused test: `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
- Layout request attribution focused test:
  `crates/fret-ui/src/tree/tests/interactive_resize_flow_rebuild.rs`
- Runtime script:
  `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-demo-with-title-toggle-underflow.json`
