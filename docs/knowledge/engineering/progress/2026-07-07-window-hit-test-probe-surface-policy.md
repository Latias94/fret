---
type: "Work Progress"
title: "Window hit-test probe surface policy"
description: "Work Progress for classifying the window hit-test probe and gating explicit manual-driver imports in the surface policy checker."
timestamp: 2026-07-07T00:51:17Z
tags: ["fret", "windowing", "hit-test", "examples", "public-surface", "source-policy"]
git_branch: "refactor/examples-probe-node-surface"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`window_hit_test_probe_demo.rs` is now part of the global public-example surface policy. It remains
an advanced/manual compatibility-driver probe, but the manual kernel/driver seams must stay explicit
instead of being hidden behind broad preludes.

# Details

- Added `apps/fret-examples/src/window_hit_test_probe_demo.rs` to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Classified the probe as `advanced_manual` under owner `examples-window-hit-test-probe`.
- Added a boundary subcheck that requires explicit `KernelApp`, `run_native_with_compat_driver`,
  and `UiAppDriver` markers.
- Rejected `advanced::prelude::*` and `component::prelude::*` in that probe surface.
- Confirmed `node_graph_demo.rs` was already globally classified as `examples-node-graph` with only
  the `fret_core` raw seam allowed.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_window_hit_test_probe_broad_manual_prelude_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_window_hit_test_probe_explicit_manual_driver_surface_is_allowed`
  failed because the scan root and boundary gate were missing.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_window_hit_test_probe_broad_manual_prelude_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_window_hit_test_probe_explicit_manual_driver_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue from the latest `main` by auditing remaining small first-party example surfaces that are
covered only by local source tests. Prefer promoting already-cleaned owner/binding boundaries into
`tools/check_surface_policy.py` before widening any raw seam allowlist.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [window_hit_test_probe_demo.rs](../../../../apps/fret-examples/src/window_hit_test_probe_demo.rs)
