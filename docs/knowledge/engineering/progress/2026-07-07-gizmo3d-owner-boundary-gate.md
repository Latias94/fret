---
type: "Work Progress"
title: "Gizmo3D owner-boundary source policy gate"
description: "Work Progress for locking Gizmo3D demo model access behind Gizmo3dDemoModelBinding in the surface policy checker."
timestamp: 2026-07-07T00:28:11Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "source-policy", "raw-model"]
git_branch: "refactor/examples-public-surface-followup"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy"
---

# Summary

`tools/check_surface_policy.py` now has a `gizmo3d` owner-boundary gate. The
advanced/manual classification may still allow raw runtime and runner seams for the proof, but
app/driver model writes must stay behind named `Gizmo3dDemoModelBinding` methods.

# Details

- Added an `examples-gizmo3d` source-policy subcheck that requires the binding and its viewport /
  frame-render entry points to remain present.
- Rejected direct `state.demo.update(app, ...)`, `demo.update(app, ...)`, and
  `model.update(app, ...)` calls from app/driver code.
- Kept `self.update(app, ...)` legal inside `Gizmo3dDemoModelBinding`; that is the owner
  mechanism, not the app-facing surface.
- Added checker fixtures for both the forbidden legacy bypass and the allowed binding-owned shape.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_gizmo3d_direct_demo_model_updates_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_gizmo3d_binding_owner_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Use the same pattern only for demos where a real owner/binding boundary has already landed. Do not
paper over broad raw seams with path allowlists; first move the mutation or output model behind a
named owner, then add a source-policy gate for that owner.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
