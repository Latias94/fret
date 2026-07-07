---
type: "Work Progress"
title: "Table stress surface policy"
description: "Work Progress for gating table stress demo model state behind TableStressControls and TableStressModelOwner."
timestamp: 2026-07-07T01:17:59Z
tags: ["fret", "table", "examples", "internal-harness", "source-policy", "raw-model"]
git_branch: "refactor/table-stress-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`table_stress_demo.rs` is now classified as an internal harness and included in the public example
scan roots. Its startup allocation, command writes, render subscriptions, and readout snapshots
must stay behind `TableStressControls` and `TableStressModelOwner`.

# Details

- Added the demo to `PUBLIC_EXAMPLE_SCAN_ROOTS` so raw table/perf seams cannot disappear from the
  global scan.
- Classified it with the same internal harness posture as `plot_stress_demo.rs`,
  `chart_stress_demo.rs`, and `virtual_list_stress_demo.rs`.
- Added a `TableStressControls`/`TableStressModelOwner` boundary subcheck to
  `tools/check_surface_policy.py`.
- Required production-source markers for model allocation, command toggles, retained table model
  exposure, render subscriptions, and readout snapshots through the controls/owner boundary.
- Rejected direct production-source `models_mut().insert(...)`, `models_mut().update(...)`,
  `update_any(...)`, UFCS `ModelStore::update(...)`, direct legacy state model references, and
  legacy `TableStressDriver::*` command helpers.
- The checker scans only production source before `#[cfg(test)]`, preserving low-level model usage
  inside source-local unit tests.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_table_stress_direct_model_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_table_stress_controls_surface_is_allowed`
  failed because the demo was not scanned and no owner-boundary violations were reported.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_table_stress_direct_model_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_table_stress_controls_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue promoting cleaned but locally guarded app/demo state boundaries into the global policy.
`editor_notes_demo.rs` is the next likely candidate because it now has `EditorAssetModels`,
`EditorNotesModelOwner`, and `EditorThemePresetBinding` but still needs a global owner-boundary
subcheck.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [table_stress_demo.rs](../../../../apps/fret-examples/src/table_stress_demo.rs)
