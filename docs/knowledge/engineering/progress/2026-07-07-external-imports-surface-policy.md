---
type: "Work Progress"
title: "External imports surface policy"
description: "Work Progress for classifying external import demos and gating shared visibility writes behind ExternalImportsModelOwner."
timestamp: 2026-07-07T00:43:56Z
tags: ["fret", "external-imports", "examples", "public-surface", "source-policy", "raw-model"]
git_branch: "refactor/examples-external-imports-surface"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

The external texture/video import demos are now part of the global public-example surface policy.
They remain advanced/manual GPU interop proofs, but visibility writes must stay behind the shared
private `ExternalImportsModelOwner` helper.

# Details

- Added the four external import demos to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Classified the native texture, web texture, AVFoundation video, and Media Foundation video demos
  as `advanced_manual` under owner `examples-external-imports`.
- Classified `external_imports_owner.rs` as an internal harness helper for shared raw
  `ModelStore` writes.
- Added an owner-boundary subcheck that rejects direct `models_mut().update(...)` and
  `ModelStore::update(...)` visibility writes in the demos while allowing
  `self.models.update(...)` inside `ExternalImportsModelOwner`.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_external_imports_direct_visibility_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_external_imports_owner_surface_is_allowed`
  failed because the scan roots and owner-boundary gate were missing.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_external_imports_direct_visibility_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_external_imports_owner_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Keep external import demos in the advanced/manual lane until a public external-import starter or
binding owns render-target setup, imported viewport plumbing, visibility state, and diagnostics.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [external_imports_owner.rs](../../../../apps/fret-examples/src/external_imports_owner.rs)
