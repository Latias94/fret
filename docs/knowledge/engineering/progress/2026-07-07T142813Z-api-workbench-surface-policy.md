---
type: Work Progress
title: API Workbench model owner boundary enters surface policy
tags:
  - fret
  - api-workbench
  - surface-policy
  - model-owner
timestamp: 2026-07-07T14:28:13Z
---

# Summary

Promoted the `api_workbench_lite_demo.rs` local owner-boundary source test into the global surface
policy gate. API Workbench remains a comparison/reference surface, but its raw `ModelStore`,
`LocalStateTxn`, query, and mutation bridge points now have a repo-level guard that keeps them
inside `ApiWorkbenchModelOwner`.

# Changed Files

- `tools/check_surface_policy.py`: names the API Workbench owner constant and adds
  `comparison-surface-api-workbench-model-owner-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for direct `models_mut().read/update`,
  UFCS `ModelStore::read/update`, owner-external `LocalStateTxn::with_model_store`, direct
  `ModelStore` aliases, and an allowed owner-routed fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_api_workbench_direct_model_access_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_api_workbench_owner_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples api_workbench_lite_demo_uses_app_local_state_and_explicit_shadcn_imports --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
