---
type: Work Progress
title: Phase 3 U13 form raw helper shrink
tags: fret,phase3,u13,cookbook,app-facade,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 removes the remaining no-new-API raw model-store availability read from
`apps/fret-cookbook/examples/form_basics.rs`.

`form_basics` already computes `can_submit` during render from the app-facing `LocalState` layout
values. The submit command availability closure now captures that frame-derived boolean instead of
reopening `host.models_mut()` through `LocalStateModelStoreExt`.

The cookbook source tests now guard against reintroducing `LocalStateModelStoreExt`,
`fret_runtime::ModelStore`, or `read_in(models...)` in the form example. The source-policy
quarantine record for `form_basics.rs` has been shrunk to the remaining `fret_ui` mechanism seam.

# Verification

Passed on 2026-07-03:

- `cargo check -p fret-cookbook --example form_basics --features cookbook-state`
- `cargo nextest run -p fret-cookbook migrated_basics_examples_use_the_new_app_surface --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all`

# Remaining U13 Work

`router_basics` still uses `RouterUiStore::{back_on_action, forward_on_action}` through the raw
action notify bridge. That should be addressed by an app-facing router binding helper or explicit
comparison/advanced classification rather than by moving `fret` dependencies into
`fret-router-ui`.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Cookbook raw helper shrink](2026-07-03-phase3-u13-cookbook-raw-helper-shrink.md)
