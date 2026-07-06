---
type: "Work Progress"
title: "Canvas DataGrid stress controls"
description: "Work Progress for bundling retained controls in the canvas datagrid stress demo."
timestamp: 2026-07-06T21:43:17Z
tags: ["fret", "examples", "data-grid", "public-surface", "raw-model", "controls"]
git_branch: "refactor/canvas-datagrid-stress-controls"
verified_by: "cargo nextest run -p fret-examples --test canvas_datagrid_stress_demo_surface --no-fail-fast"
---

# Summary

`canvas_datagrid_stress_demo.rs` now keeps its retained stress controls behind
`CanvasDataGridStressControls` instead of exposing `variable_sizes`, `clamp_rows`, and `revision`
as separate raw model fields on the window state.

# Details

- Added `CanvasDataGridStressControls` for variable row/column sizing, row clamping, and the
  measurement revision.
- Added `CanvasDataGridStressControlsSnapshot` so render observes and reads the retained controls
  through one named bundle.
- Kept `grid_output` on the app-facing `LocalState<DataGridCanvasOutput>` path from the earlier
  data-grid output bridge cleanup.
- Extended `canvas_datagrid_stress_demo_surface.rs` to require the controls bundle and reject
  direct window-state model fields, scattered startup inserts, and direct render subscriptions.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test canvas_datagrid_stress_demo_surface canvas_datagrid_stress_demo_bundles_stress_controls --no-fail-fast`
  failed because `CanvasDataGridStressControls` did not exist.
- `cargo fmt --all --check`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test canvas_datagrid_stress_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue with the higher-ROI 3D cleanup:

- add a `Plot3dSurfaceBinding` / `Plot3dViewportBinding` for `plot3d_demo.rs`;
- after that, migrate the plot portion of `gizmo3d_demo.rs` before tackling the larger
  `Gizmo3dDemoModel` mutation surface.

# Citations

- [canvas_datagrid_stress_demo.rs](../../../../apps/fret-examples/src/canvas_datagrid_stress_demo.rs)
- [canvas_datagrid_stress_demo_surface.rs](../../../../apps/fret-examples/tests/canvas_datagrid_stress_demo_surface.rs)
