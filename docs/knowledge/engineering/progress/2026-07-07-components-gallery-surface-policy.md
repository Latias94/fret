---
type: "Work Progress"
title: "Components gallery surface policy"
description: "Work Progress for gating components gallery model allocation and driver writes behind ComponentsGalleryModelBundle/Owner."
timestamp: 2026-07-07T00:59:18Z
tags: ["fret", "components-gallery", "examples", "public-surface", "source-policy", "raw-model"]
git_branch: "refactor/components-gallery-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`components_gallery.rs` already had demo-local owner helpers and source tests. The global surface
policy now enforces that same boundary: startup model allocation must stay behind
`ComponentsGalleryModelBundle`, and app/driver model writes must stay behind
`ComponentsGalleryModelOwner`.

# Details

- Reused the existing `examples-components-gallery` advanced/manual classification.
- Added a `components_gallery` owner-boundary subcheck to `tools/check_surface_policy.py`.
- Required production-source markers for `ComponentsGalleryModelBundle`, `ComponentsGalleryModelOwner`,
  and the command helper wrappers.
- Rejected direct production-source `models_mut().update(...)`, `models_mut().insert(...)`, and
  `ModelStore::update(...)`/`update_any(...)` bypasses.
- The checker scans only production source before `#[cfg(test)]` for this boundary so source-test
  marker strings do not satisfy or trip the gate.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_components_gallery_direct_model_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_components_gallery_owner_surface_is_allowed`
  failed because zero owner-boundary violations were reported.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_components_gallery_direct_model_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_components_gallery_owner_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Promote the remaining already-cleaned stress/editor surfaces in the same way, starting with
`virtual_list_stress_demo.rs` or `table_stress_demo.rs`, because they now have named controls
bindings and local source tests but no global owner-boundary subcheck yet.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [components_gallery.rs](../../../../apps/fret-examples/src/components_gallery.rs)
