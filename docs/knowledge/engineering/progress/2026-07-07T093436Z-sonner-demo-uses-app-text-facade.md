---
type: "Work Progress"
title: "Sonner demo uses app text facade"
description: "Work Progress for Sonner demo uses app text facade."
timestamp: 2026-07-07T09:34:36Z
tags: ["ui-surface", "examples", "sonner", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/sonner-demo-text-facade"
---

# Summary

Moved `sonner_demo.rs` header title and status readout text off
`fret_ui_kit::declarative::text` and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/sonner_demo.rs`
- `apps/fret-examples/tests/sonner_demo_surface.rs`

Decision:

- Keep the toast demo's manual runner, `UiTree`, and shadcn `Sonner` command wiring unchanged.
- Replace only the ordinary app text roles with `fret::app::text::section_chrome_label` and
  `fret::app::text::control_readout`, then update the surface test to reject the older `decl_text`
  teaching seam.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test sonner_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- direct text helper scan for `sonner_demo.rs` found no `decl_text`,
  `fret_ui_kit::declarative::text`, `text_section_chrome_label`, or `text_control_readout` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with the next public example
surface where an app facade can replace direct component helper imports without hiding intentional
runner ownership.

# Citations

- `apps/fret-examples/src/sonner_demo.rs`
- `apps/fret-examples/tests/sonner_demo_surface.rs`
