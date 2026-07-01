---
type: Work Progress
title: U9 view scheduling and layout-query split
tags: fret,u9,facade,scheduling,layout-query,modularity
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The app-facing scheduling and layout-query helpers moved from `ecosystem/fret/src/view.rs` into
`ecosystem/fret/src/view/scheduling.rs` and `ecosystem/fret/src/view/layout_query.rs`.

The public `AppUi` methods remain unchanged: `request_animation_frame`, `set_continuous_frames`,
`layout_query_bounds`, `layout_query_region_with_id`, `layout_query_region`, and
`environment_viewport_bounds`. Source-shape tests and render-authoring capability tests aggregate
the new modules so the app capability contract follows the internal split.

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers app_ui_keeps_command_gating_and_animation_frame_surface_without_deref --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U9 by splitting the remaining lane-barrier methods and core AppUi shell helpers, or pause
the facade split after the current modularity checkpoint.
