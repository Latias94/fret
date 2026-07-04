---
type: Work Progress
title: Phase 3 U13 pointer drag facade
tags: fret,phase3,u13,pointer,cookbook,surface-policy
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 now has an app-facing pointer-region authoring lane for custom low-level pointer
streams.

- Added `fret::pointer` as an explicit opt-in module, not part of `fret::app::prelude::*`.
- Added `PointerRegion` and `PointerActionCx` wrappers over the mechanism-level pointer region and
  action host. App code can use capture/release, cursor changes, focus-default suppression,
  LocalState reads/writes, redraw/notify, and paint/layout invalidation without importing
  `fret_ui::action::UiPointerActionHost`, `PointerRegionProps`, `DefaultAction`, or raw
  `Model<T>`.
- Migrated `apps/fret-cookbook/examples/drag_basics.rs` from the advanced lane to the default app
  lane: `LocalState<T>` in `View::init`, `fret::pointer::{...}` for pointer streams, and
  `cx.pointer_region(...)` instead of `cx.elements().pointer_region(...)`.
- Moved `drag_basics.rs` from `ADVANCED_MANUAL_SURFACES` to `DEFAULT_AUTHORING_SURFACES` and added
  source-policy rules that reject raw pointer-region mechanisms in default app/tutorial surfaces.
- Updated `docs/crate-usage-guide.md` to describe `fret::pointer` as the explicit pointer lane.

# Verification

- `cargo check -p fret --lib`
- `cargo check -p fret-cookbook --example drag_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret usage_docs_prefer_grouped_app_ui_actions root_surface_exposes_explicit_style_and_icon_modules app_prelude_omits_low_level_mechanism_types app_prelude_pub_use_budget_is_curated_and_closed --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib migrated_basics_examples_use_the_new_app_surface advanced_examples_use_the_explicit_advanced_surface advanced_interaction_examples_keep_pointer_region_on_explicit_elements_escape_hatch selected_cookbook_examples_prefer_handle_first_tracked_reads --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next

Continue U13 with the remaining public-looking advanced surfaces. The current order is:

1. `async_inbox_basics.rs`: needs an app-facing background job / inbox action helper before it can
   leave quarantine.
2. `canvas_pan_zoom_basics.rs`: should reuse the pointer facade, then add a narrow canvas authoring
   facade for painter/geometry seams.
3. `chart_interactions_basics.rs`: defer until chart command/state binding replaces the remaining
   retained tree and command registry seams.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Cookbook drag example](../../../apps/fret-cookbook/examples/drag_basics.rs)
- [Fret pointer facade](../../../ecosystem/fret/src/view/pointer.rs)
- [Surface policy gate](../../../tools/check_surface_policy.py)
