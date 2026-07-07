---
type: "Work Progress"
title: "Form demo uses app text facade"
description: "Work Progress for Form demo uses app text facade."
timestamp: 2026-07-07T09:27:20Z
tags: ["ui-surface", "examples", "form", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/form-demo-text-facade"
---

# Summary

Moved `form_demo.rs` header status readout text off `fret_ui_kit::declarative::text` and onto the
app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/form_demo.rs`
- `apps/fret-examples/tests/form_demo_surface.rs`

Decision:

- Keep the demo's manual runner and `UiTree` ownership unchanged; this slice only removes the
  unnecessary component text helper leak from the app render surface.
- Update the surface test to require `fret::app::text::control_readout` and reject the older
  `decl_text`/`text_control_readout` teaching seam.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test form_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- direct text helper scan for `form_demo.rs` found no `decl_text`,
  `fret_ui_kit::declarative::text`, or `text_control_readout` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with the next low-risk public
example text facade cleanup.

# Citations

- `apps/fret-examples/src/form_demo.rs`
- `apps/fret-examples/tests/form_demo_surface.rs`
