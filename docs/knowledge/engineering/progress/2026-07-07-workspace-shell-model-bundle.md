---
type: "Work Progress"
title: "Workspace shell model bundle"
description: "Work Progress for workspace shell model bundle cleanup."
timestamp: 2026-07-07T00:15:00Z
tags: ["fret", "examples", "workspace-shell", "public-surface", "raw-model"]
git_branch: "refactor/workspace-shell-model-bundle"
verified_by: "cargo nextest run -p fret-examples --test workspace_shell_driver_state_surface --no-fail-fast"
---

# Summary

`workspace_shell_demo/driver.rs` now keeps startup shared-model allocation behind a private
`WorkspaceShellModelBundle`.

# Details

- Added `WorkspaceShellModelBundle::new(...)` to own the demo's initial `ModelStore::insert` calls.
- Changed `WorkspaceShellDemoDriver::build_ui(...)` so window setup constructs a named model bundle
  instead of scattering direct `app.models_mut().insert(...)` calls.
- Strengthened `workspace_shell_driver_state_surface` so production source must keep update/set
  writes behind `WorkspaceShellModelOwner` and startup inserts behind `WorkspaceShellModelBundle`.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test workspace_shell_driver_state_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue with plot/chart family binding cleanup or another first-contact advanced surface.

# Citations

- [driver.rs](../../../../apps/fret-examples/src/workspace_shell_demo/driver.rs)
- [workspace_shell_driver_state_surface.rs](../../../../apps/fret-examples/tests/workspace_shell_driver_state_surface.rs)
