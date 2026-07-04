---
type: Work Progress
title: Phase 3 U13 async inbox facade
tags: fret,phase3,u13,async,cookbook,surface-policy
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 now has an explicit app-facing async/background-work lane for cookbook authoring.

- Added `fret::async_work` as an explicit opt-in module, not part of `fret::app::prelude::*`.
- Added `AppAsyncWorkExt`, `InboxLocal<T>`, `AppInboxCx`, `inbox_local(...)`, and
  `inbox_drain_apply(...)` so default app code can register inbox drainers and update
  `LocalState<T>` from runner-drained messages without importing `fret_runtime::Model`,
  `ModelStore`, `InboxDrainRegistry`, or `AppUiRawActionNotifyExt`.
- Kept concrete executor choice outside the default facade: cookbook async still imports
  `fret-executor` explicitly for `Inbox`, `InboxDrainer`, `Executors`, `BackgroundTask`, and
  cancellation primitives.
- Migrated `apps/fret-cookbook/examples/async_inbox_basics.rs` to `LocalState<T>`,
  `cx.actions().local(...)`, grouped `locals_with(...)` actions, `ui::children!`, and
  `shadcn` controls that accept app-facing local state.
- Moved `async_inbox_basics.rs` from the advanced manual quarantine list to default clean
  source-policy coverage.

# Verification

- `cargo check -p fret --lib`
- `cargo check -p fret-cookbook --features cookbook-async --example async_inbox_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo check -p fret-cookbook --features cookbook-async --all-targets`
- `cargo nextest run -p fret usage_docs_prefer_grouped_app_ui_actions root_surface_exposes_explicit_style_and_icon_modules app_prelude_omits_low_level_mechanism_types app_prelude_pub_use_budget_is_curated_and_closed --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib migrated_basics_examples_use_the_new_app_surface advanced_examples_use_the_explicit_advanced_surface advanced_interaction_examples_keep_pointer_region_on_explicit_elements_escape_hatch cookbook_examples_limit_raw_action_notify_to_host_owned_cases selected_cookbook_examples_prefer_handle_first_tracked_reads --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib --no-fail-fast`
- `cargo nextest run -p fret inbox_drain_apply_updates_local_state_and_requests_one_redraw --no-fail-fast`
- `cargo nextest run -p fret --lib --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `cargo fmt --all --check`
- `git diff --check`

# Next

Continue U13 with `canvas_pan_zoom_basics.rs`. It can reuse `fret::pointer`, but still needs a
narrow canvas authoring facade for painter/geometry seams before leaving quarantine.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Cookbook async inbox example](../../../apps/fret-cookbook/examples/async_inbox_basics.rs)
- [Fret async-work facade](../../../ecosystem/fret/src/view/async_work.rs)
- [Surface policy gate](../../../tools/check_surface_policy.py)
