---
type: "Work Progress"
title: "Memory and perf harness surface policy"
description: "Work Progress for classifying image-heavy, text-heavy, and marquee perf demos as internal harnesses."
timestamp: 2026-07-07T02:46:04Z
tags: ["fret", "examples", "memory", "perf", "internal-harness", "source-policy"]
git_branch: "refactor/memory-perf-harness-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

The image-heavy, text-heavy, and marquee perf demo cluster is now included in public example
scanning and classified as internal harnesses. These demos deliberately apply memory, cache, atlas,
and perf pressure rather than teaching the default app facade.

# Details

- Added `image_heavy_memory_demo.rs`, `text_heavy_memory_demo.rs`, and
  `extras_marquee_perf_demo.rs` to `INTERNAL_HARNESS_SURFACES`.
- Added all three paths to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Kept per-file raw seam allowlists exact because these harnesses use different mechanisms:
  GPU image upload/drop hooks, low-level text/container rendering, and advanced `KernelApp`
  diagnostics startup.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
  failed because the memory/perf harness paths were not in `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue with large advanced proof surfaces (`genui_demo.rs`, `imui_editor_proof_demo.rs`,
`imui_node_graph_demo.rs`) or choose a facade migration candidate (`assets_demo.rs`, `query_demo.rs`,
`query_async_tokio_demo.rs`).

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
