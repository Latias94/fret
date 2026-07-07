---
type: Work Progress
title: GenUI model owner boundary enters surface policy
tags:
  - fret
  - genui
  - surface-policy
  - model-owner
timestamp: 2026-07-07T14:16:09Z
---

# Summary

Promoted the `genui_demo.rs` model-owner source test into the global surface policy gate. The GenUI
demo remains an advanced/reference integration surface, but runtime model reads and writes now have
a repo-level guard that keeps them behind `GenUiModelOwner`.

# Changed Files

- `tools/check_surface_policy.py`: names the GenUI owner constant and adds
  `advanced-surface-genui-model-owner-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for direct `models_mut().read/update*`,
  UFCS `ModelStore::read/update*`, direct `ModelStore` alias bypasses, and an allowed owner-routed
  fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples genui_demo_model_writes_stay_behind_owner_helpers --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
