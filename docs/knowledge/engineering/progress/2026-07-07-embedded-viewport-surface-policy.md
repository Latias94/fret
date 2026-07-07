---
type: "Work Progress"
title: "Embedded viewport surface policy"
description: "Work Progress for classifying the embedded viewport demo and gating its model owner boundary in the surface policy checker."
timestamp: 2026-07-07T00:37:04Z
tags: ["fret", "embedded-viewport", "examples", "public-surface", "source-policy", "raw-model"]
git_branch: "refactor/examples-public-surface-next"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`embedded_viewport_demo.rs` is now part of the global public-example surface policy. It remains an
advanced/manual interop proof, but its forwarded-input readout write is explicitly owned by
`EmbeddedViewportDemoModelOwner`.

# Details

- Added `apps/fret-examples/src/embedded_viewport_demo.rs` to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Classified the demo as `advanced_manual` with owner `examples-embedded-viewport`.
- Added an owner-boundary subcheck that requires the demo-local owner and embedded viewport hook
  markers to stay present.
- Rejected direct `models_mut().update(...)` and `ModelStore::update(...)` writes in the app/driver
  body while keeping `self.models.update(...)` legal inside `EmbeddedViewportDemoModelOwner`.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_embedded_viewport_direct_model_updates_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_embedded_viewport_owner_surface_is_allowed`
  failed because the scan root and owner-boundary gate were missing.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_embedded_viewport_direct_model_updates_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_embedded_viewport_owner_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue promoting already-cleaned example owner boundaries from local source tests into
`tools/check_surface_policy.py`, but only after each surface has a named owner/binding instead of
free helper mutation code.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [embedded_viewport_demo.rs](../../../../apps/fret-examples/src/embedded_viewport_demo.rs)
