---
type: "Work Progress"
title: "Smoke and effects demo surface policy"
description: "Work Progress for classifying effects and first-frame smoke demos outside default app authoring."
timestamp: 2026-07-07T02:19:22Z
tags: ["fret", "examples", "renderer-lab", "internal-harness", "source-policy"]
git_branch: "refactor/smoke-effects-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`effects_demo.rs` and `first_frame_smoke_demo.rs` are now included in public example scanning but
classified away from default app authoring. The effects demo is a renderer lab; the first-frame
demo is an internal launch/backend smoke harness.

# Details

- Added `effects_demo.rs` to `RENDERER_LAB_SURFACES` because it owns direct `SceneOp` effect
  composition, manual `FnDriver` hooks, GPU-ready renderer perf toggles, and env-driven profiling.
- Added `first_frame_smoke_demo.rs` to `INTERNAL_HARNESS_SURFACES` because it owns manual startup,
  render hooks, and auto-close behavior for first-frame backend validation.
- Added both paths to `PUBLIC_EXAMPLE_SCAN_ROOTS` while keeping them out of
  `ADVANCED_MANUAL_SURFACES`.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
  failed because the smoke/effects paths were not in `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_renderer_labs_do_not_count_as_advanced_manual_quarantine`
  failed because `effects_demo.rs` was not classified as a renderer lab.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_renderer_labs_do_not_count_as_advanced_manual_quarantine`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue with either renderer/media labs (`alpha_mode_demo.rs`, `image_upload_demo.rs`,
`drop_shadow_demo.rs`) or true facade-migration candidates (`assets_demo.rs`, `query_demo.rs`,
`query_async_tokio_demo.rs`).

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
