---
type: "Work Progress"
title: "Streaming import demo surface policy"
description: "Work Progress for classifying streaming image import demos as advanced public examples."
timestamp: 2026-07-07T02:11:51Z
tags: ["fret", "streaming", "examples", "advanced-surface", "source-policy"]
git_branch: "refactor/streaming-import-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

The streaming import demo cluster is now included in public example scanning and classified as
advanced/manual. These demos intentionally own low-level image registration/update effects and
manual `FnDriver` event/render hooks, so they should not be treated as default app-authoring
examples.

# Details

- Added scan roots and advanced/manual classifications for:
  `streaming_i420_demo.rs`, `streaming_image_demo.rs`, and `streaming_nv12_demo.rs`.
- Introduced `STREAMING_IMPORT_DEMO_FILENAMES` and `STREAMING_IMPORT_ALLOWED_RAW_SEAMS` to keep
  the classification exact for this cluster.
- The allowed seams are intentionally limited to:
  `fret_app`, `fret_core`, `fret_launch`, `fret_runtime`, and `FnDriver`.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
  failed because the streaming import demo paths were not in `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue classifying remaining raw/advanced example gaps. The next compact candidates are manual
smoke/probe demos such as `effects_demo.rs`, `first_frame_smoke_demo.rs`, and `alpha_mode_demo.rs`,
or the larger conformance/form/table group after a source-level read.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
