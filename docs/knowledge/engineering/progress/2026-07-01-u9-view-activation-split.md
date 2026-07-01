---
type: Work Progress
title: U9 view activation bridge split
tags: fret,u9,facade,activation,actions,modularity
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The explicit activation-only bridge moved from `ecosystem/fret/src/view.rs` into
`ecosystem/fret/src/view/activation.rs`. `view.rs` re-exports `AppActivateSurface` and
`AppActivateExt`, and keeps `action_listener` available internally for the grouped action helper.
The dispatch listener helpers remain `pub(super)` to avoid widening the public app surface.

This preserves the existing explicit bridge posture: activation-only widgets can opt into
`AppActivateExt`, while ordinary action-capable widgets should keep using native `.action(...)` /
`.action_payload(...)` slots.

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers dispatch_listener_queues_a_command_effect dispatch_payload_listener_records_payload_before_dispatch action_listener_hides_activate_reason_for_simple_widget_glue app_activate_surface_contract_can_store_activation_handlers app_activate_ext_action_alias_dispatches_without_turbofish app_activate_ext_action_payload_alias_records_payload_without_turbofish --no-fail-fast`
- `cargo nextest run -p fret --test app_render_actions_surface --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Split the remaining grouped action helper carriers and `AppRenderActionsExt` implementation into
`view/actions.rs`.
