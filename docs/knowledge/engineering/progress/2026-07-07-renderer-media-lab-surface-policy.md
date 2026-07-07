---
type: "Work Progress"
title: "Renderer media lab surface policy"
description: "Work Progress for classifying alpha, image upload, and drop shadow demos as renderer labs."
timestamp: 2026-07-07T02:27:17Z
tags: ["fret", "examples", "renderer-lab", "media", "source-policy"]
git_branch: "refactor/renderer-media-lab-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

The alpha-mode, image-upload, and drop-shadow demo cluster is now included in public example
scanning and classified as renderer labs. These demos validate renderer/media semantics and
diagnostic behavior rather than teaching the default app facade.

# Details

- Added `alpha_mode_demo.rs` to `RENDERER_LAB_SURFACES` for straight-vs-premultiplied alpha upload
  and image compositing semantics.
- Added `image_upload_demo.rs` to `RENDERER_LAB_SURFACES` for keyed image asset upload, eviction,
  and direct scene image rendering.
- Added `drop_shadow_demo.rs` to `RENDERER_LAB_SURFACES` for `DropShadowV1` renderer semantics and
  perf-baseline validation.
- Added all three paths to `PUBLIC_EXAMPLE_SCAN_ROOTS` with per-file exact raw seam allowlists.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
  failed because the renderer media lab paths were not in `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_renderer_labs_do_not_count_as_advanced_manual_quarantine`
  failed because the renderer media lab paths were not classified under `RENDERER_LAB_SURFACES`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_renderer_labs_do_not_count_as_advanced_manual_quarantine`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue with smoke/conformance harnesses or choose a real facade migration candidate such as
`assets_demo.rs`, `query_demo.rs`, or `query_async_tokio_demo.rs`.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
