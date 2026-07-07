---
type: "Work Progress"
title: "Docking demo uses app text facade"
description: "Work Progress for Docking demo uses app text facade."
timestamp: 2026-07-07T10:41:26Z
tags: ["ui-surface", "examples", "docking", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/docking-demo-text-facade"
---

# Summary

Moved `docking_demo.rs` hierarchy row labels and inspector readouts off raw
`fret_ui_kit::declarative::text` imports and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/docking_demo.rs`
- `apps/fret-examples/tests/docking_demo_surface.rs`

Decision:

- Keep docking runtime setup, panel registry behavior, dev-state export, overlay hooks, and
  diagnostic anchor semantics unchanged.
- Preserve `docking_demo_diagnostic_anchor` as a raw `UiHost` helper because it emits diagnostics
  semantics anchors, not text-role chrome.
- Convert only `docking_demo_list_row_text` and `docking_demo_readout_text` to
  `AppRenderContext<'a>` and call `text::list_row_label` / `text::control_readout`.
- Extend the surface test to require the app text facade and forbid the old raw `decl_text` text
  constructor path for panel labels/readouts.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test docking_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with neighboring docking text
facade seams such as docking arbitration and container-query docking.

# Citations

- `apps/fret-examples/src/docking_demo.rs`
- `apps/fret-examples/tests/docking_demo_surface.rs`
