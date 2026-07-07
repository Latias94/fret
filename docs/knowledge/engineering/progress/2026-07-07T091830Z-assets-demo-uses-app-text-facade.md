---
type: "Work Progress"
title: "Assets demo uses app text facade"
description: "Work Progress for Assets demo uses app text facade."
timestamp: 2026-07-07T09:18:30Z
tags: ["ui-surface", "examples", "assets", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/assets-demo-text-facade"
---

# Summary

Moved `assets_demo.rs` stats readout text off `fret_ui_kit::declarative::text` and onto the
app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/tests/assets_demo_surface.rs`

Decision:

- Keep the SVG/GPU-ready asset wiring explicit; this slice only removes the unnecessary component
  text helper leak from the app render surface.
- Reuse existing `fret::app::text::control_readout`, then keep the demo-specific muted foreground
  as inherited styling.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test assets_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- direct text helper scan for `assets_demo.rs` found no `decl_text` or
  `fret_ui_kit::declarative::text` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with larger advanced/manual
surfaces only where the next raw seam can be shrunk without hiding deliberate renderer or runner
ownership.

# Citations

- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/tests/assets_demo_surface.rs`
