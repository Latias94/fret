---
type: Work Progress
title: U9 view runtime split
tags: fret,u9,facade,view-runtime,modularity
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The `ecosystem/fret` view runtime cluster moved from `ecosystem/fret/src/view.rs` into the private
`ecosystem/fret/src/view/runtime.rs` module. `view.rs` remains the aggregator and re-exports
`ViewWindowState`, `AppUiRenderRootState`, `view_init_window`, `view_view`,
`render_root_with_app_ui`, and desktop `view_record_engine_frame`, so existing
`crate::view::*` and `fret::advanced::view::*` paths stay intact.

`view_authoring_api_source()` now includes `view/runtime.rs` alongside `view/context.rs` and
`view/local_state.rs`, keeping source-shape tests contract-oriented as internal modules continue to
split.

# Changed Files

- `ecosystem/fret/src/view/runtime.rs`
- `ecosystem/fret/src/view.rs`

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo check -p fret --locked --features batteries`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers view_runtime_exposes_only_app_ui_as_the_public_context_name manual_render_root_with_app_ui_keeps_handlers_and_local_state_alive locals_with_runtime_dispatch_updates_locals_and_rerenders_cached_view view_runtime_cache_enable_transition_keeps_toggle_group_footer_semantics_after_compact_resize payload_models_runtime_dispatch_updates_shared_models_and_requests_redraw --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U9 by deciding whether the next low-risk split should isolate the AppUi actions/data/effects
helper groups or first move component/recipe adapter code out of `view/local_state.rs`.
