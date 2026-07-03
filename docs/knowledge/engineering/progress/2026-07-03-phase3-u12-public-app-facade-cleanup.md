---
type: Work Progress
title: Phase 3 U12 public app facade cleanup
tags: fret,phase3,u12,app-facade,cookbook,surface-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U12 moved selected default cookbook and starter-adjacent authoring paths off public-looking
raw model/action seams.

- Added `AppLocalStateExt::local_state(value)` on the default `fret::app` facade so `View::init`
  can create `LocalState<T>` handles without spelling `LocalState::new_in(app.models_mut(), ...)`.
- Added app-facing shadcn/Sonner effect helpers for default toast, success toast, and dismiss-all
  flows.
- Migrated `data_table_basics.rs`, `toast_basics.rs`, and `hello_counter.rs` to use
  `fret::app`/`fret::style` facade lanes instead of raw local-state construction, raw action
  notification hooks, or direct `fret_core` imports.
- Extended `tools/check_surface_policy.py` so selected default cookbook examples are now scanned
  for `fret::advanced`, raw action notify hooks, `LocalState::new_in`, `ModelStore`,
  direct `fret_core`, direct `fret_ui`, `UiTree`, and `FnDriver` seams.
- Updated crate usage and golden-path docs so default authoring teaches `app.local_state(value)`
  plus `cx.actions().transient(...)` / `cx.effects()` instead of raw host hooks.

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
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `cargo fmt --all --check`
- `git diff --check`
- Static `rg` over selected default cookbook examples for raw seams returned no matches.

# Remaining Work

U13 should split or shrink the remaining advanced facade lanes and quarantine records now that more
default cookbook paths have app-facing replacements. The data-table example still owns a
`DataTableViewOutput` model handle through the shadcn table API; treat that as a table-output API
follow-up rather than a default local-state constructor seam.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Surface policy gate](../../../tools/check_surface_policy.py)
- [Cookbook contract tests](../../../apps/fret-cookbook/src/lib.rs)
- [App facade](../../../ecosystem/fret/src/lib.rs)
