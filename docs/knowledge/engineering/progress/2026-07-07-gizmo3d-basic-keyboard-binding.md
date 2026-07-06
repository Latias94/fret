---
type: "Work Progress"
title: "Gizmo3D basic keyboard binding"
description: "Work Progress for routing basic Gizmo3D keyboard mutations through binding methods."
timestamp: 2026-07-06T22:39:09Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "keyboard", "binding"]
git_branch: "refactor/gizmo3d-command-owner"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now routes the basic keyboard mutation cluster through named
`Gizmo3dDemoModelBinding` methods instead of opening `state.demo.update(...)` closures directly in
the event branches.

# Details

- Added `cancel_active_or_in_progress(...)` for Esc handling.
- Added `set_transform_mode(...)` for T/R/S/U shortcuts, preserving op-mask-aware behavior.
- Added `toggle_help(...)`, `toggle_op_mask(...)`, `toggle_depth_mode(...)`, and
  `toggle_universal_translate_depth(...)`.
- Updated the keyboard event branches to call those methods while preserving the existing redraw
  behavior.
- Extended `gizmo3d_demo_surface.rs` to require those binding calls and reject the old direct event
  branch model writes.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_basic_keyboard_mutations_through_binding --no-fail-fast`
  failed because the binding methods were not used.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue with the visual preset / size-policy keyboard cluster, then selection/camera shortcuts.
Viewport-input routing and render-HUD updates should remain separate slices because they interact
with undo records, tool arbitration, and paint-time overlay state.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
