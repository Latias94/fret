---
type: "Work Progress"
title: "Gizmo3D model binding"
description: "Work Progress for hiding the Gizmo3D demo's shared model handle behind a demo-local binding."
timestamp: 2026-07-06T22:28:15Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "raw-model", "binding"]
git_branch: "refactor/gizmo3d-model-owner"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now stores and registers `Gizmo3dDemoModelBinding` instead of exposing
`fret_runtime::Model<Gizmo3dDemoModel>` directly in the window state and per-window service.

# Details

- Added `Gizmo3dDemoModelBinding` as the demo-local owner for the shared Gizmo3D model handle.
- Moved model allocation behind `Gizmo3dDemoModelBinding::new(...)`.
- Moved startup viewport-theme application behind `apply_viewport_theme(...)`.
- Moved viewport target and size updates behind `sync_viewport_target(...)`, with a read-before-write
  check so steady-state frames do not dirty the shared demo model.
- Updated `Gizmo3dDemoService` to store the binding instead of a raw model handle.
- Extended `gizmo3d_demo_surface.rs` to forbid raw `Gizmo3dDemoModel` handles in window/service
  state and direct startup allocation.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_hides_demo_model_handle_behind_binding --no-fail-fast`
  failed because `Gizmo3dDemoModelBinding` did not exist.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Convert the largest `state.demo.update(...)` clusters in event/command handling into semantic
methods on `Gizmo3dDemoModelBinding` or a narrow companion owner. This slice intentionally keeps
generic `read(...)` / `update(...)` as a transition layer so behavior stays stable while the raw
model handle is removed from the app-facing surface.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
