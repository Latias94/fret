---
type: "Work Progress"
title: "Custom effect reference demo surface policy"
description: "Work Progress for classifying custom-effect reference demos as advanced public examples."
timestamp: 2026-07-07T02:06:51Z
tags: ["fret", "custom-effects", "examples", "advanced-surface", "source-policy"]
git_branch: "refactor/custom-effect-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

The custom-effect reference demo cluster is now included in public example scanning and classified
as advanced/manual. These demos intentionally validate renderer/effect ABI and custom-effect
contract behavior instead of teaching the default app-authoring facade.

# Details

- Added scan roots and advanced/manual classifications for:
  `custom_effect_v1_demo.rs`, `custom_effect_v2_demo.rs`, `custom_effect_v3_demo.rs`, and
  `custom_effect_v3_web_demo.rs`.
- Introduced `CUSTOM_EFFECT_REFERENCE_DEMO_FILENAMES` plus separate allowed raw seam sets for:
  native reference demos and the manual web runner demo.
- Kept `custom_effect_v3_web_demo.rs` separate from the existing custom-effect v2 web parameter
  binding cluster because it owns direct scene/effect composition and manual `FnDriver` web runner
  hooks, not the shared v2 web owner/control-binding surface.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
  failed because the custom-effect reference demo paths were not in `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue classifying remaining raw/advanced example gaps. The next compact cluster is likely the
streaming import demo set: `streaming_i420_demo.rs`, `streaming_image_demo.rs`, and
`streaming_nv12_demo.rs`.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
