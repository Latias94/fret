---
type: "Work Progress"
title: "Plot3D panel binding"
description: "Work Progress for adding an app-facing Plot3D binding and migrating plot3d_demo."
timestamp: 2026-07-06T22:02:11Z
tags: ["fret", "plot3d", "examples", "public-surface", "raw-model", "binding"]
git_branch: "refactor/plot3d-surface-binding"
verified_by: "cargo nextest run -p fret-plot3d --no-fail-fast"
---

# Summary

`fret-plot3d` now exposes `Plot3dPanelBinding`, and `plot3d_demo.rs` uses it instead of storing and
mutating a raw `fret_runtime::Model<Plot3dModel>` in app code.

# Details

- Added `Plot3dPanelBinding` as the app-facing owner for the Plot3D panel model.
- Added `viewport_untracked(...)` for render-target allocation code that runs outside UI render.
- Added `sync_viewport_target(...)` so engine-owned target identity and pixel size update through a
  named Plot3D surface instead of direct model writes. The sync path reads first and only updates on
  real changes, so steady-state engine frames do not dirty the Plot3D model.
- Migrated `plot3d_demo.rs` to store `Plot3dPanelBinding` and build `plot3d_panel(...)` from
  `state.plot.panel_props()`.
- Added `plot3d_demo_surface.rs` to reject raw `Plot3dModel` handles and manual
  `Plot3dPanelProps::new(state.plot.clone())` in the demo.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test plot3d_demo_surface plot3d_demo_uses_app_facing_plot3d_binding --no-fail-fast`
  failed because `Plot3dPanelBinding` was not imported or used.
- `cargo nextest run -p fret-plot3d --no-fail-fast`
- `cargo nextest run -p fret-examples --test plot3d_demo_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Migrate the plot panel portion of `gizmo3d_demo.rs` to `Plot3dPanelBinding`, then handle the larger
`Gizmo3dDemoModel` mutation surface with a dedicated owner/binding pass.

# Citations

- [binding.rs](../../../../ecosystem/fret-plot3d/src/binding.rs)
- [plot3d_demo.rs](../../../../apps/fret-examples/src/plot3d_demo.rs)
- [plot3d_demo_surface.rs](../../../../apps/fret-examples/tests/plot3d_demo_surface.rs)
