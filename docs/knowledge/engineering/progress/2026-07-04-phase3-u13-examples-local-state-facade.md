---
type: Work Progress
title: Phase 3 U13 fret-examples local-state facade migration
tags: fret,phase3,u13,examples,app-surface,local-state
timestamp: 2026-07-04
related_plan: ../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

This slice removes the remaining default-example `LocalState::new_in(app.models_mut(), ...)`
constructor leaks from selected `apps/fret-examples/src` demos and routes them through the
app-facing `AppLocalStateExt::local_state(...)` constructor.

Migrated:

- `async_playground_demo.rs`
- `datatable_demo.rs`
- `form_demo.rs`
- `genui_demo.rs`
- `launcher_utility_window_demo.rs`
- `launcher_utility_window_materials_demo.rs`
- `table_demo.rs`

The examples now import `fret::app::AppLocalStateExt as _` where needed instead of teaching a raw
`ModelStore` constructor at initialization sites.

# Policy Updates

Updated `tools/examples_source_tree_policy/gate.py` and
`tools/examples_source_tree_policy/grouped_state.py` so the required markers use
`app.local_state(...)` and the old `LocalState::new_in(app.models_mut(), ...)` spelling is a
forbidden marker for these default/example surfaces.

# Verification

Passed:

- `cargo check -p fret-examples`
- `cargo fmt --all --check`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`
- `PYTHONPATH=tools python3 -m py_compile tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/grouped_state.py`
- Targeted static search for `LocalState::new_in(app.models_mut(), ...)` in the migrated files
  returns no matches.

Known unrelated gate drift:

- `PYTHONPATH=tools python3 tools/examples_source_tree_policy/gate.py` still reports 32 pre-existing
  marker failures in other examples such as `simple_todo_demo.rs`, `hello_counter_demo.rs`,
  `todo_demo.rs`, `api_workbench_lite_demo.rs`, and `imui_editor_proof_demo.rs`. None of the
  migrated files above are listed in that failure output.

# Next Action

Continue Phase 3 closeout classification. Remaining `LocalState::new_in` matches are either
advanced/manual helpers, test evidence, policy forbidden markers, or older docs/workstream history;
they should not be treated as default example guidance unless a fresh closeout search finds a new
normal app-facing use.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
