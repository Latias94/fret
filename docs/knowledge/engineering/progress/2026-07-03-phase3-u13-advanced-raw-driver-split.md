---
type: Work Progress
title: Phase 3 U13 advanced raw and driver split
tags: fret,phase3,u13,advanced-facade,raw,driver
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 first slice split the `fret::advanced` facade without taking on the broader cookbook
classification migration.

- Added `fret::advanced::raw` for raw action/model hooks:
  `AppUiRawActionNotifyExt`, `AppUiRawModelExt`, `LocalStateRawModelExt`,
  `LocalStateModelStoreExt`, and `LocalStateElementContextExt`.
- Added `fret::advanced::driver` for driver/builder escape hatches:
  `ui_app*`, `run_native_with_fn_driver*`, `FretAppAdvancedExt`,
  `UiAppBuilderAdvancedExt`, `UiAppBuilder`, `UiAppDriver`, and `ViewElements`.
- Removed the `advanced::prelude` wildcard export of `advanced::*`; the prelude still carries
  advanced view-authoring conveniences but no longer imports raw traits by accident.
- Updated cookbook raw call sites, docs, tests, and source-policy fixtures to import raw hooks from
  `fret::advanced::raw`.

# Verification

Passed on 2026-07-03:

- `cargo nextest run -p fret --lib --no-fail-fast`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret-cookbook --lib --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `cargo fmt --all --check`
- `git diff --check`
- Static old raw import search returned no matches:
  `rg -n "use fret::advanced::(AppUiRaw|LocalStateRaw|LocalStateModel|LocalStateElement)|fret::advanced::AppUiRaw|fret::advanced::LocalState(Model|Raw|Element)" ecosystem/fret docs apps tools -g '*.rs' -g '*.md' -g '*.py'`

# Remaining Work

Continue U13 by migrating cookbook/examples away from `advanced::prelude::*` where they only need
default/app-facing or explicit view/driver lanes, then tighten `tools/check_surface_policy.py`
quarantine categories and allowed raw seam accounting.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Advanced facade](../../../ecosystem/fret/src/lib.rs)
- [U13 subagent audits](../subagents/2026-07-03-phase3-u13-advanced-facade-audits.md)

