---
type: "Work Progress"
title: "Canvas datagrid stress surface policy"
description: "Work Progress for gating canvas datagrid stress telemetry and retained controls behind LocalState and CanvasDataGridStressControls."
timestamp: 2026-07-07T01:40:47Z
tags: ["fret", "canvas-datagrid", "examples", "internal-harness", "source-policy", "raw-model"]
git_branch: "refactor/canvas-datagrid-stress-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`canvas_datagrid_stress_demo.rs` is now classified as an internal harness and included in public
example scanning. Its grid telemetry must stay on app-facing `LocalState`, and its retained stress
control models must stay bundled behind `CanvasDataGridStressControls`.

# Details

- Added the demo to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Classified it with the other stress/perf harnesses instead of treating it as a copyable default
  app-authoring surface.
- Added a `CanvasDataGridStressControls` boundary subcheck to `tools/check_surface_policy.py`.
- Required compact production-source markers for `LocalState` grid output, `app.local_state(...)`,
  `local_state_txn(...)`, layout reads, `DataGrid::output_model(...)`, and axis construction through
  the controls snapshot revision.
- Rejected legacy raw `Model<DataGridCanvasOutput>` output plumbing, direct stress-control model
  fields on `CanvasDataGridStressWindowState`, direct variable/clamp/revision model allocation
  outside the controls bundle, and old direct `&state.variable_sizes`/`&state.clamp_rows`/
  `&state.revision` reads.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_canvas_datagrid_stress_raw_control_plumbing_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_canvas_datagrid_stress_controls_surface_is_allowed`
  failed because the demo was not scanned/classified and no controls-boundary violations were
  reported.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_canvas_datagrid_stress_raw_control_plumbing_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_canvas_datagrid_stress_controls_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue reducing the remaining unclassified public example scan gaps. The next broad cluster is
manual chart/plot demo surfaces that own `FnDriver`/`UiTree` seams but are still absent from
`PUBLIC_EXAMPLE_SCAN_ROOTS`.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [canvas_datagrid_stress_demo.rs](../../../../apps/fret-examples/src/canvas_datagrid_stress_demo.rs)
