---
type: "Work Progress"
title: "Text input conformance surface policy"
description: "Work Progress for classifying CJK, emoji, and IME demos as internal conformance harnesses."
timestamp: 2026-07-07T02:40:40Z
tags: ["fret", "examples", "text", "ime", "conformance", "source-policy"]
git_branch: "refactor/text-input-conformance-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

The CJK, emoji, and IME demo cluster is now included in public example scanning and classified as
internal conformance harnesses. These surfaces validate text layout, font fallback, emoji rendering,
IME event behavior, hot reload, and direct retained `UiTree` frame rendering rather than teaching
the default app facade.

# Details

- Added `cjk_conformance_demo.rs`, `emoji_conformance_demo.rs`, and `ime_smoke_demo.rs` to
  `INTERNAL_HARNESS_SURFACES`.
- Added all three paths to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Kept per-file raw seam allowlists exact, because the emoji and IME harnesses use
  `fret::advanced` while the CJK harness does not.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
  failed because the text/input conformance paths were not in `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue with memory/perf harness classification (`image_heavy_memory_demo.rs`,
`text_heavy_memory_demo.rs`, `extras_marquee_perf_demo.rs`) or start a real facade migration
candidate (`assets_demo.rs`, `query_demo.rs`, `query_async_tokio_demo.rs`).

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
