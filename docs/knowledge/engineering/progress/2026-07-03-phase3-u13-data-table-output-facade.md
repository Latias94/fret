---
type: Work Progress
title: Phase 3 U13 data table output facade cleanup
tags: fret,phase3,u13,data-table,cookbook,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 seventeenth slice closes the `data_table_basics.rs` default-surface blind spot where
the example was classified as default clean but still imported `fret_runtime::Model` for
`DataTableViewOutput`.

# Changes

- Added `IntoTableViewOutputModel` beside `IntoTableStateModel` in the table declarative layer.
- Updated shadcn `DataTable::output_model(...)` and `DataTablePagination::new(...)` to accept the
  narrow table-output adapter while continuing to store `Model<TableViewOutput>` internally.
- Implemented the adapter for `fret::app::LocalState<TableViewOutput>`.
- Added `DataTable::into_element_in(...)` so default `AppUi` callers can land the table through
  `ElementContextAccess` without calling `cx.elements()`.
- Migrated `data_table_basics.rs` to `LocalState<shadcn::DataTableViewOutput>` and
  `.into_element_in(...)`.
- Tightened `tools/check_surface_policy.py` so default app/tutorial surfaces reject
  `fret_runtime::` imports, and adjusted the fretboard scaffold source assertion to avoid
  tripping the new gate.

# Rationale

The right fix is not a generic `IntoModel<T>` escape hatch. Data table output telemetry is a
specific recipe contract, so it gets a specific adapter. This keeps default cookbook code on
`LocalState` while preserving raw `Model<TableViewOutput>` for lower-level component and test
surfaces that already own runtime state directly.

# Verification

Passed:

- `cargo check -p fret-cookbook --features cookbook-table --example data_table_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret-cookbook --lib cookbook_data_table_example_prefers_local_state_table_bridges --no-fail-fast`
- `cargo nextest run -p fret-ui-kit --lib table_surfaces_keep_a_narrow_table_state_bridge --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --lib data_table_surfaces_keep_narrow_table_state_bridges selected_public_model_backed_seams_stay_on_audited_allowlist --no-fail-fast`
- `cargo nextest run -p fretboard --lib mutation_workbench_template_uses_public_app_facade_only --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U13 by tackling the remaining cookbook quarantine records with real facade gaps. Highest
value candidates are IMUI action/editor command/local-state adapters, then the true
host/effect-driven `async_inbox_basics.rs` raw action case.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Readonly U13 cookbook quarantine audit](../subagents/2026-07-03-phase3-u13-cookbook-quarantine-readonly-audit.md)
