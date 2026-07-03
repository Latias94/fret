---
type: Work Progress
title: Phase 2 U14 AppUi facade narrowing
tags: fret,phase2,u14,appui,facade,local-state,public-surface
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f257e-be4e-72a2-8f6b-3dfc2b88bca8
---

# Phase 2 U14 AppUi Facade Narrowing

## Summary

Phase 2 U14 narrows the default app authoring surface by moving raw `LocalState<T>` bridges out of
the primary `local_state.rs` implementation and behind explicit advanced extension traits. The
default `fret::app::prelude::*` stays focused on `LocalState<T>`, grouped `cx.state()`,
`cx.actions()`, and `cx.data()` helpers, while raw `Model<T>`, `ModelStore`, and direct
`ElementContext` bridge methods require importing the advanced lane.

This slice is intentionally breaking for manual examples that called `read_in` as an inherent
method. Those call sites now import `fret::advanced::LocalStateModelStoreExt as _`, making the raw
store dependency visible.

## Changes

- Added `ecosystem/fret/src/view/local_state/bridges.rs` for:
  - `LocalStateRawModelExt`
  - `LocalStateModelStoreExt`
  - `LocalStateElementContextExt`
- Moved `from_model`, `model`, `clone_model`, `read_in`, `revision_in`, `value_in*`,
  `update_in*`, `set_in`, and `*_in` `ElementContext` helpers from inherent `LocalState<T>` methods
  into the explicit traits.
- Added `ecosystem/fret/src/view/local_state/adapters.rs` for component/model adapter impls while
  preserving `LocalState` component ergonomics such as shadcn model inputs.
- Added `ecosystem/fret/src/view/data/render.rs` for `AppRenderData` / `AppRenderDataExt`, reducing
  the `view/data.rs` aggregator pressure without changing grouped helper names.
- Re-exported the new bridge traits from `fret::advanced` and kept them out of
  `fret::app::prelude::*`.
- Updated facade/source-shape tests so future growth must keep `view/data.rs` and
  `view/local_state.rs` split instead of regrowing monolithic files.
- Updated cookbook examples that intentionally read through a raw `ModelStore` to import the
  advanced trait explicitly.
- Marked the `hello_counter` cookbook example as requiring `cookbook-assets`, matching its
  `fret::icons::icon` dependency and keeping `cargo check -p fret-cookbook --all-targets` honest.

## Verification

Verification passed before commit:

- `cargo check -p fret --all-targets`
- `cargo nextest run -p fret data_and_local_state_modules_stay_split_instead_of_regrowing_aggregators local_state_owner_module_stays_private_with_view_reexports grouped_authoring_surfaces_replace_flat_app_ui_helpers local_state_docs_classify_default_and_bridge_surfaces advanced_prelude_reexports_app_facing_view_aliases app_prelude_omits_low_level_mechanism_types advanced_prelude raw_state app_render_data --no-fail-fast`
- `cargo nextest run -p fret --lib --no-fail-fast`
- `cargo check -p fret-examples-imui --all-targets`
- `cargo check -p fret-cookbook --all-targets`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `git diff --check`

## Next Action

Commit U14, then run the Phase 2 closeout pass: verify U1-U14 status against the plan definition of
done, record retained bridges that are intentionally outside this plan, and decide whether any final
dead migration code or stale tests should be removed before marking the goal complete.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Fret app facade](../../../../ecosystem/fret/src/lib.rs)
- [View module index](../../../../ecosystem/fret/src/view.rs)
- [LocalState bridge traits](../../../../ecosystem/fret/src/view/local_state/bridges.rs)
- [AppRenderData split](../../../../ecosystem/fret/src/view/data/render.rs)
