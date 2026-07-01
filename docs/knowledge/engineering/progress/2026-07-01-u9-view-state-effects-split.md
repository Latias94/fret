---
type: Work Progress
title: U9 view state and effects split
tags: fret,u9,facade,appui,state,effects,modularity
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The grouped `AppUi::state()` and `AppUi::effects()` carrier types moved out of
`ecosystem/fret/src/view.rs` into `view/state.rs` and `view/effects.rs`. `view.rs` continues to
re-export `AppUiState` and `AppUiEffects`, and `view_authoring_api_source()` now aggregates the new
modules for source-shape tests.

This is a deliberately small U9 modularity cut before the larger action/data helper split. It keeps
the public `cx.state()` / `cx.effects()` authoring surface unchanged while shrinking the facade
monolith.

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers local_state_value_in_helpers_clone_store_values local_state_from_model_wraps_existing_raw_handle local_state_new_in_allocates_without_exposing_raw_model_handle local_state_borrowed_read_helpers_project_without_clone_noise local_state_bridge_read_helpers_project_without_clone_noise --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test app_render_actions_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Split the larger AppUi action helper group into `view/actions.rs`, using constructors or
`pub(super)` fields for parent-module construction while preserving `AppRenderActionsExt` and the
explicit `AppActivateExt` bridge surface.
