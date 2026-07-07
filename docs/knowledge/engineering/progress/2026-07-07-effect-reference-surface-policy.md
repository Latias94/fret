---
type: "Work Progress"
title: "Effect reference surface policy"
description: "Work Progress for classifying liquid-glass and postprocess theme demos as advanced references."
timestamp: 2026-07-07T02:55:55Z
tags: ["fret", "examples", "effects", "advanced-surface", "source-policy"]
git_branch: "refactor/effect-reference-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

The liquid-glass and postprocess-theme demo cluster is now included in public example scanning and
classified as advanced/manual. Both files explicitly describe themselves as advanced/reference
surfaces and validate renderer/effect contracts rather than teaching default app authoring.

# Details

- Added `liquid_glass_demo.rs` to `ADVANCED_MANUAL_SURFACES` for backdrop warp/custom-effect
  capability validation and explicit effect/control graph ownership.
- Added `postprocess_theme_demo.rs` to `ADVANCED_MANUAL_SURFACES` for renderer/theme bridge
  custom-effect composition and high-ceiling postprocess controls.
- Added both paths to `PUBLIC_EXAMPLE_SCAN_ROOTS` with exact raw seam allowlists.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
  failed because the effect reference paths were not in `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue with large proof surfaces (`genui_demo.rs`, `imui_editor_proof_demo.rs`,
`imui_node_graph_demo.rs`) or migrate real default-facade candidates (`assets_demo.rs`,
`query_demo.rs`, `query_async_tokio_demo.rs`).

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
