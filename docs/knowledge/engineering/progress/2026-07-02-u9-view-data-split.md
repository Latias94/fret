---
type: Work Progress
title: U9 view data helpers split
tags: fret,u9,facade,data,modularity
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The grouped data/query/mutation helper surface moved from `ecosystem/fret/src/view.rs` into
`ecosystem/fret/src/view/data.rs`: `AppUiData`, `AppRenderData`, `AppRenderDataExt`, query and
mutation read extension traits, selector input traits, selector tuple implementations, and the
private query/mutation helper functions.

`view.rs` remains the public facade aggregator and keeps `AppUi::data()` on the `AppUi` shell, while
re-exporting the same public app names. Feature-gated imports in `view/data.rs` keep the backend-free
`fret --no-default-features --features app` profile warning-free. The source-shape tests now aggregate
`view/data.rs`, and the app-render action/data surface integration tests aggregate their extracted
modules so they do not self-pass from test assertion literals inside `view.rs`.

The mutation helper tests also now use the explicit `cx.elements().container(...)` escape hatch for
their empty element shell, matching the AppUi lane-sealing contract instead of relying on an
unavailable direct `cx.container(...)` method.

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo check -p fret --locked`
- `cargo check -p fret --locked --features batteries`
- `cargo check -p fret --locked --no-default-features --features app,state-query,state-mutation`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers --no-fail-fast`
- `cargo nextest run -p fret --features state-mutation app_ui_data --no-fail-fast`
- `cargo nextest run -p fret --test app_render_data_surface --no-fail-fast`
- `cargo nextest run -p fret --test app_render_actions_surface --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U9 by splitting the remaining raw/advanced escape hatches and AppUi shell helpers into
narrow modules without widening the default app authoring lane.
