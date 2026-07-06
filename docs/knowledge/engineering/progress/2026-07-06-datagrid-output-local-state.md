---
type: Work Progress
title: DataGrid canvas output LocalState bridge
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/raw-surface-contracts
tags: fret,ui-framework,public-surface,data-grid,local-state,raw-model
---

# Summary

`DataGridCanvas::output_model(...)` now accepts a dedicated
`IntoDataGridCanvasOutputModel` bridge instead of forcing callers to pass a raw
`Model<DataGridCanvasOutput>`.

The bridge is intentionally narrow:

- `fret-ui-shadcn` still stores `Option<Model<DataGridCanvasOutput>>` internally because the canvas
  grid publishes telemetry through the runtime model graph;
- raw `Model<DataGridCanvasOutput>` and `&Model<DataGridCanvasOutput>` remain valid for low-level
  and manual surfaces;
- `fret::app::LocalState<DataGridCanvasOutput>` now implements the bridge so app-facing examples
  can share telemetry output without exposing raw model plumbing.

`canvas_datagrid_stress_demo.rs` now uses `LocalState<shadcn::DataGridCanvasOutput>` for
`grid_output`. The stress harness still keeps raw models for stress controls and revision state; the
output handle no longer has to be raw just because the harness is performance-oriented.

# Decision

This follows the existing `DataTable` output pattern rather than introducing a generic
`IntoModel<T>` abstraction. Data-grid output telemetry is a named component contract, so a named
bridge keeps the public API discoverable while preserving the internal retained model mechanism.

# Verification

- Red first:
  `cargo nextest run -p fret-ui-shadcn data_grid_canvas_output_uses_narrow_output_bridge shadcn_component_surfaces_keep_expected_raw_model_seams --no-fail-fast`
- Green after implementation:
  `cargo nextest run -p fret-ui-shadcn --lib data_grid_canvas_output_uses_narrow_output_bridge selected_public_model_backed_seams_stay_on_audited_allowlist --no-fail-fast`
  `cargo nextest run -p fret-examples --test canvas_datagrid_stress_demo_surface --no-fail-fast`
  `cargo check -p fret-examples --lib --tests`
- `python3 tools/gate_table_source_policy.py`

# Next

`table_stress_demo.rs` now has a named local stress contract through `TableStressControls`. Keep it
on the retained stress/perf path rather than mechanically converting it to `LocalState`, and use
similarly narrow contracts for future data-grid stress controls.
