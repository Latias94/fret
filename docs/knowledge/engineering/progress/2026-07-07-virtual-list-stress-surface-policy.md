---
type: "Work Progress"
title: "Virtual list stress surface policy"
description: "Work Progress for gating virtual-list stress demo model state behind VirtualListStressControls."
timestamp: 2026-07-07T01:09:55Z
tags: ["fret", "virtual-list", "examples", "internal-harness", "source-policy", "raw-model"]
git_branch: "refactor/virtual-list-stress-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`virtual_list_stress_demo.rs` is now classified as an internal harness and included in the public
example scan roots. Its shared model allocation, command writes, and render snapshot reads must stay
behind `VirtualListStressControls`.

# Details

- Added the demo to `PUBLIC_EXAMPLE_SCAN_ROOTS` so raw seams cannot disappear from the global scan.
- Classified it with the same internal harness posture as `plot_stress_demo.rs` and
  `chart_stress_demo.rs`: it is a pressure/perf harness, not a copyable default app authoring
  surface.
- Added a `VirtualListStressControls` boundary subcheck to `tools/check_surface_policy.py`.
- Required production-source markers for startup allocation, command toggles, and render snapshot
  reads through `VirtualListStressControls`.
- Rejected direct production-source `models_mut().insert(...)`, `models_mut().update(...)`,
  `update_any(...)`, UFCS `ModelStore::update(...)`, and legacy owner/free-helper bypasses.
- The checker scans only production source before `#[cfg(test)]`, so local unit tests can still use
  low-level model APIs as fixtures without weakening the app/harness boundary.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_virtual_list_stress_direct_model_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_virtual_list_stress_controls_surface_is_allowed`
  failed because the demo was not scanned and no owner-boundary violations were reported.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_virtual_list_stress_direct_model_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_virtual_list_stress_controls_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Promote the remaining cleaned but locally guarded stress/editor demos into the global policy, with
`table_stress_demo.rs` and `editor_notes_demo.rs` as the next likely candidates.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [virtual_list_stress_demo.rs](../../../../apps/fret-examples/src/virtual_list_stress_demo.rs)
