---
type: Work Progress
title: Phase 4 U6 raw surface shrink
timestamp: 2026-07-04T17:15:06Z
git_branch: feat/ui-framework-phase2-refactor
related_plan: docs/plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
tags: fret,ui-framework,phase4,u6,public-surface,raw-seams
---

# Summary

Phase 4 U6 shrank the public advanced raw surface after the default app replacements were already
available.

The `fret::advanced::prelude::*` wildcard no longer exports raw retained-tree or raw model tracked
read seams. `UiTree`, raw `Model`/`ModelStore` types, `TrackedModelExt`, raw action/model traits,
and the raw `LocalState` constructor now live on `fret::advanced::raw`.

# Decisions

- `LocalState::new_in(...)` is now crate-internal. Public app code should use
  `AppLocalStateExt::local_state(...)` during `View::init`, and manual/raw code should use
  `fret::advanced::raw::local_state_in(...)`.
- `TrackedModelExt` is treated as a raw `Model<T>` helper for the `fret` facade. It remains
  available under `fret::advanced::raw` instead of being pulled in by `advanced::prelude::*`.
- Cookbook manual examples that still need `UiTree` import it from `fret::advanced::raw::UiTree`.
- `query_status_badge_for(&state)` was added so the default cookbook query example does not teach
  `cx.elements()` only to create a status badge.

# Verification

- `cargo nextest run -p fret --lib --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn state_helpers_prefer_typed_badge_outputs_when_no_runtime_landing_seam_is_required --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --features state -E 'test(query_badge_adapter_maps_status_and_error_alert_without_state_stack_leakage)' --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib --no-fail-fast`
- `cargo check -p fret-cookbook --examples`
- `cargo check -p fret-cookbook --example query_basics --features cookbook-query`
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `git diff --check`

# Follow-Up

- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` remains an explicit
  advanced/raw snippet. If it is meant to be copyable first-contact material, classify it in the
  surface policy gate or migrate it to a default app-facing helper.
- `fret-bootstrap` rustdoc still leads with direct-driver examples before the `ui_app(...)` golden
  path; this is a documentation ordering cleanup, not a raw surface blocker.

