---
type: Work Progress
title: Workspace shell driver owner boundary enters surface policy
tags:
  - fret
  - workspace-shell
  - surface-policy
  - model-owner
timestamp: 2026-07-07T14:09:51Z
---

# Summary

Promoted the `workspace_shell_demo/driver.rs` owner-boundary source test into the global surface
policy gate. The workspace shell remains an advanced/manual proof surface, but its driver model
allocation and writes now have a repo-level guard that keeps them behind
`WorkspaceShellModelBundle` and `WorkspaceShellModelOwner`.

# Changed Files

- `tools/check_surface_policy.py`: names the workspace-shell owner constant and adds
  `advanced-surface-workspace-shell-driver-owner-boundary`, scoped to
  `workspace_shell_demo/driver.rs` inside the directory-level advanced/manual classification.
- `tools/test_check_surface_policy.py`: adds fixture coverage for direct `models_mut().update*`,
  UFCS `ModelStore::update*`, direct `models_mut().insert(...)`, and direct `ModelStore` alias
  bypasses, plus an allowed owner-routed fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples workspace_shell_driver_model_writes_stay_behind_owner_helpers --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
