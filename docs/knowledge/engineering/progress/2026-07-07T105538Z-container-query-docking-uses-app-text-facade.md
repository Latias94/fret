---
type: "Work Progress"
title: "Container-query docking demo uses app text facade"
description: "Work Progress for Container-query docking demo uses app text facade."
timestamp: 2026-07-07T10:55:38Z
tags: ["ui-surface", "examples", "docking", "container-queries", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/container-query-docking-text-facade"
---

# Summary

Moved `container_queries_docking_demo.rs` panel readout and placeholder text helpers off raw
`fret_ui_kit::declarative::text` calls and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/container_queries_docking_demo.rs`
- `apps/fret-examples/tests/container_queries_docking_surface.rs`

Decision:

- Keep docking runtime setup, container query region behavior, split-handle diagnostics, and panel
  registry behavior unchanged.
- Convert `container_query_docking_readout_text` and
  `container_query_docking_placeholder_text` to `AppRenderContext<'a>`.
- Narrow `DemoDockPanelRegistry::render_left_panel` from a generic `UiHost` helper to the actual
  `ElementContext<'_, App>` host used by `DockPanelElementRegistry<App>`, allowing the app text
  facade to cover fixed panel text.
- Preserve the diagnostic anchor helper as a raw `UiHost` seam because it emits test semantics, not
  text-role chrome.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test container_queries_docking_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with remaining app text facade
seams that do not require new facade roles.

# Citations

- `apps/fret-examples/src/container_queries_docking_demo.rs`
- `apps/fret-examples/tests/container_queries_docking_surface.rs`
