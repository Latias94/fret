---
type: "Work Progress"
title: "Docking arbitration demo uses app text facade"
description: "Work Progress for Docking arbitration demo uses app text facade."
timestamp: 2026-07-07T10:49:33Z
tags: ["ui-surface", "examples", "docking", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/docking-arbitration-text-facade"
---

# Summary

Moved `docking_arbitration_demo.rs` fixed state readouts and body paragraph helper off raw
`fret_ui_kit::declarative::text` calls and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `apps/fret-examples/tests/docking_arbitration_surface.rs`

Decision:

- Keep docking arbitration policy, viewport panel roots, overlay/modal state, diagnostics, and model
  controls unchanged.
- Preserve raw `ElementContext<'_, App>` seams for the advanced docking/viewport render path.
- Convert only `docking_arbitration_readout_text` and `docking_arbitration_paragraph_text` to
  `AppRenderContext<'a>` and call `text::control_readout` / `text::paragraph`.
- Extend the surface test to require the app text facade and forbid the old raw kit text constructor
  path for state readouts and body text.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test docking_arbitration_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with neighboring text facade
seams such as container-query docking.

# Citations

- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `apps/fret-examples/tests/docking_arbitration_surface.rs`
