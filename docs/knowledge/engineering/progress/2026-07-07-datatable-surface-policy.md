---
type: "Work Progress"
title: "Datatable surface policy"
description: "Work Progress for gating datatable demo output on app-facing LocalState."
timestamp: 2026-07-07T01:47:40Z
tags: ["fret", "datatable", "examples", "advanced-surface", "source-policy", "raw-model"]
git_branch: "refactor/datatable-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`datatable_demo.rs` is now included in public example scanning and classified as an advanced/manual
example. Its `DataTableViewOutput` handle must stay on app-facing `LocalState`, not raw shared model
plumbing.

# Details

- Added the demo to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Classified it as advanced/manual because it still owns manual driver/table harness plumbing.
- Added a datatable output-boundary subcheck to `tools/check_surface_policy.py`.
- Required compact production-source markers for `LocalState<shadcn::DataTableViewOutput>`,
  `app.local_state(...)`, `table_output.layout_value(cx)`, `DataTablePagination`, and
  `DataTable::output_model(...)`.
- Rejected legacy `Model<shadcn::DataTableViewOutput>`, raw output model insertion,
  `cx.observe_model(&table_output, Invalidation::Layout)`, and the old `fret_app::Model` import.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_datatable_raw_output_model_plumbing_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_datatable_local_state_output_surface_is_allowed`
  failed because the demo was not scanned/classified and no output-boundary violations were
  reported.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_datatable_raw_output_model_plumbing_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_datatable_local_state_output_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue with the remaining unclassified manual chart/plot demos that own `FnDriver`/`UiTree`
surfaces but are still absent from `PUBLIC_EXAMPLE_SCAN_ROOTS`.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [datatable_demo.rs](../../../../apps/fret-examples/src/datatable_demo.rs)
