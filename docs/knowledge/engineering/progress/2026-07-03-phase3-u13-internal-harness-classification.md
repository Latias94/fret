---
type: Work Progress
title: Phase 3 U13 internal harness classification
tags: fret,phase3,u13,source-policy,fret-examples,internal-harness
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 eleventh slice makes `internal_harness` a real source-policy category instead of a
recognized-but-unused label.

# Changes

- Added `INTERNAL_HARNESS_SURFACES` to `tools/check_surface_policy.py`.
- `docking_arbitration_demo.rs` and `plot_stress_demo.rs` moved from `advanced_manual` retirement
  records to `internal_harness` records.
- `check_surface_policy(...)` now validates and scans internal harness records for owner and
  allowed raw seams, and public example raw-seam discovery treats those records as classified.
- Source-policy tests now cover internal harness allowed/unlisted seam behavior and assert
  `plot_stress_demo.rs` is no longer an `advanced_manual` surface.

# Rationale

`advanced_manual` means a public-looking surface is temporarily quarantined and should eventually
retire through public wrappers or a cleaner split. Stress and conformance harnesses are different:
they intentionally own manual driver/runtime seams as infrastructure. Keeping them in
`advanced_manual` created false retirement pressure and made the quarantine ledger less useful.

# Verification

Passed:

- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U13 by moving more true stress/conformance-only files into `internal_harness`, or migrate
copyable `advanced_manual` examples toward app-facing wrappers.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [`fret-examples` precise quarantine](2026-07-03-phase3-u13-fret-examples-precise-quarantine.md)
- [`fret-examples` explicit raw imports](2026-07-03-phase3-u13-fret-examples-raw-imports.md)
