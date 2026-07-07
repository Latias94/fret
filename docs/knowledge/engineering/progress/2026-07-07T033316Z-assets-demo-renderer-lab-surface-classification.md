---
type: "Work Progress"
title: "Assets demo renderer lab surface classification"
description: "Work Progress for Assets demo renderer lab surface classification."
timestamp: 2026-07-07T03:33:16Z
tags: ["ui-surface", "assets", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
---

# Summary

Classified `apps/fret-examples/src/assets_demo.rs` as a renderer lab instead of a default-clean app
example. The demo is intentionally retained as an ADR/cache probe for `UiAppDriver`
`ImageAssetCache` / `SvgAssetCache` GPU-ready integration and advanced SVG registration hooks.
The copyable asset authoring story remains in cookbook assets examples.

# Details

Changed files:

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Policy change:

- Added `assets_demo.rs` to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Added `assets_demo.rs` to `RENDERER_LAB_SURFACES` with owner
  `examples-assets-cache-renderer-lab`.
- Listed only the raw seam the source-policy scanner sees today: `fret::advanced`.
- Added tests that keep the demo in renderer-lab classification and out of advanced/manual
  quarantine.

Verification passed before commit:

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- Raw public example inventory script now reports 15 remaining uncovered files.

# Next Action

Continue with one of the remaining 15 public-example raw-surface gaps. Good next slices:

- `async_playground_demo.rs`: likely default-facade migration for text helpers, but scroll/pressable
  raw seams need a source read before classification.
- Utility/window demos: likely advanced/manual classification because they own manual window
  lifecycle and launcher interop.

# Citations

- `docs/knowledge/engineering/subagents/2026-07-07-public-example-surface-followup-audit.md`
- `docs/workstreams/open-source-readiness-fearless-refactor-v1/M4_CANDIDATES.md`
- `apps/fret-examples/tests/assets_demo_surface.rs`
