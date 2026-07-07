---
type: "Work Progress"
title: "Virtual list stress demo uses app text facade"
description: "Work Progress for Virtual list stress demo uses app text facade."
timestamp: 2026-07-07T10:36:05Z
tags: ["ui-surface", "examples", "virtual-list", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/virtual-list-stress-text-facade"
---

# Summary

Moved `virtual_list_stress_demo.rs` header and row-label text helpers off raw
`fret_ui_kit::declarative::text` imports and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/virtual_list_stress_demo.rs`
- `apps/fret-examples/tests/virtual_list_stress_demo_surface.rs`

Decision:

- Keep the virtual-list render root, keyed virtualization path, scroll handle, model controls, and
  performance reporting unchanged.
- Preserve the existing `ElementContext<'_, App>` layout snapshot seam because it is part of the
  manual stress harness path, not a text-role helper.
- Convert only `virtual_list_stress_readout_text` and `virtual_list_stress_row_label_text` to
  `AppRenderContext<'a>` and call `text::control_readout` / `text::list_row_label`.
- Extend the surface test to require the app text facade and forbid the old raw `decl_text` text
  constructor path for header and row text.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test virtual_list_stress_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with remaining default app text
facade seams such as docking examples where the current app facade already has matching roles.

# Citations

- `apps/fret-examples/src/virtual_list_stress_demo.rs`
- `apps/fret-examples/tests/virtual_list_stress_demo_surface.rs`
