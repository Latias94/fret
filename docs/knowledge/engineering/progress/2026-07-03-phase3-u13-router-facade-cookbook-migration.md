---
type: Work Progress
title: Phase 3 U13 router facade cookbook migration
tags: fret,phase3,u13,router,cookbook,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 thirteenth slice removes the raw action-notify bridge from the default router
cookbook by adding an app-facing router history binding helper.

# Changes

- Added `fret::router::bind_history_actions(...)` for typed back/forward action binding on the
  default `AppUi` lane.
- Migrated `router_basics.rs` off `fret::advanced::raw::AppUiRawActionNotifyExt` and
  `fret_ui::CommandAvailability`.
- Fixed `router_basics.rs` under the `cookbook-router` feature by adapting existing router
  link/outlet helpers through explicit `cx.elements()` calls.
- Moved `router_basics.rs` from `ADVANCED_MANUAL_SURFACES` to `DEFAULT_AUTHORING_SURFACES`.
- Updated usage docs and facade/source-policy tests to prefer the new router helper and reject the
  old raw import on default cookbook paths.

# Rationale

`router_basics.rs` is copyable default cookbook material. Its only advanced classification reason
was the missing app-facing wrapper for `RouterUiStore::{back_on_action, forward_on_action}`. The
new helper keeps raw host-facing registration inside the facade while preserving explicit opt-in to
the router ecosystem.

# Verification

Passed:

- `cargo check -p fret-cookbook --features cookbook-router --example router_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret --lib readme_and_rustdoc_expose_router_as_explicit_optional_surface usage_docs_expose_router_as_explicit_extension_surface root_surface_module_budget_is_curated_and_closed --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue U13 with either `virtual_list_basics.rs` facade cleanup or the `apps/fret-examples/src/lib.rs`
classification move into `internal_harness`.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Command facade cookbook migration](2026-07-03-phase3-u13-command-facade-cookbook-migration.md)
