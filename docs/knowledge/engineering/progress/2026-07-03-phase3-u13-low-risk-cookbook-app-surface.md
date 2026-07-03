---
type: Work Progress
title: Phase 3 U13 low-risk cookbook app surface migration
tags: fret,phase3,u13,cookbook,app-facade,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 third slice migrates the remaining low-risk asset/effect cookbook examples that do not
need raw runtime seams.

Changed examples:

- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
- `apps/fret-cookbook/examples/effects_layer_basics.rs`
- `apps/fret-cookbook/examples/app_owned_bundle_assets_basics.rs`
- `apps/fret-cookbook/examples/icons_and_assets_basics.rs`

`assets_reload_epoch_basics` now uses `App`, `WindowId`, app prelude imports, app-facing
`cx.request_animation_frame()`, and `IntoUiElement<App>` helper returns instead of `KernelApp`,
`advanced::{KernelApp, prelude::Effect}`, or `Effect::RequestAnimationFrame`.

`effects_layer_basics` now uses `LocalState<Option<Arc<str>>>` from the app facade and
`app.local_state(...)` in `View::init`, removing the previous `advanced::prelude::*` and raw
`Model` teaching surface.

The app-owned bundle and icons/assets examples were also tightened to call asset source-state
helpers through `cx.elements()`. This keeps the examples on `AppComponentCx` helper boundaries and
fixes the hidden `cookbook-assets` feature build path after their previous migration.

# Tests And Gates

Passed on 2026-07-03:

- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `cargo check -p fret-cookbook --all-targets`
- `cargo check -p fret-cookbook --all-targets --features cookbook-assets`
- `cargo check -p fret-examples-imui --all-targets`
- `cargo nextest run -p fret-cookbook --lib --no-fail-fast`
- Static search over the four touched cookbook examples for `KernelApp`, `AppWindowId`,
  `advanced::prelude`, `advanced::raw`, `use fret::advanced`, `Model<`,
  `IntoUiElement<KernelApp>`, and `Effect::RequestAnimationFrame` returned no matches.

# Remaining U13 Work

U13 is still open. The next high-value slice is to make source policy discover public-looking raw
seams that are not covered by `DEFAULT_SURFACES` or `ADVANCED_MANUAL_SURFACES`, then classify them
as default/app, advanced driver/view/interop/raw, internal harness, or migration reference.

Useful follow-on candidates from the latest read-only audits:

- migrate simple no-new-API helpers such as `paint_value_in` / `layout_value_in` to public helper
  forms where feasible;
- move remaining cookbook availability and virtual-list raw state reads through `locals_with` or
  app-facing captures;
- keep true driver, view, interop, and raw examples explicit instead of using
  `advanced::prelude::*`;
- shrink the broad `apps/fret-examples-imui/src` quarantine into per-file records once raw seam
  categories are in the gate.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [U13 cookbook and IMUI quarantine tightening](2026-07-03-phase3-u13-cookbook-imui-quarantine.md)
- [U13 advanced facade audits](../subagents/2026-07-03-phase3-u13-advanced-facade-audits.md)
