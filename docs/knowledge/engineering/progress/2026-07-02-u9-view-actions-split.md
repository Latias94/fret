---
type: Work Progress
title: U9 view action helpers split
tags: fret,u9,facade,actions,modularity
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The grouped app action helper carriers moved from `ecosystem/fret/src/view.rs` into
`ecosystem/fret/src/view/actions.rs`: `AppUiActions`, `AppUiActionLocal`, `AppUiLocalsWith`,
`AppRenderActions`, `AppRenderActionLocal`, `AppRenderLocalsWith`, the extracted render action hook
owner, and `AppRenderActionsExt`.

`view.rs` still re-exports the same public app facade names, so default `fret` users keep the same
`cx.actions()` and prelude/root imports while the internal authoring facade is now narrow enough to
continue splitting data/query/effects/raw seams without one large view file hiding contract drift.
The source-shape tests now aggregate `view/actions.rs` so future internal movement remains checked
against the public authoring surface instead of a physical file layout.

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers locals_with_runtime_dispatch_updates_locals_and_rerenders_cached_view payload_models_runtime_dispatch_updates_shared_models_and_requests_redraw app_activate_ext_action_alias_dispatches_without_turbofish app_activate_ext_action_payload_alias_records_payload_without_turbofish dispatch_listener_queues_a_command_effect dispatch_payload_listener_records_payload_before_dispatch --no-fail-fast`
- `cargo nextest run -p fret --test app_render_actions_surface --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U9 by splitting grouped data/query/mutation helpers out of `view.rs`, preserving the same
public `AppUiData` / `AppRenderData` names and source-shape guarantees.
