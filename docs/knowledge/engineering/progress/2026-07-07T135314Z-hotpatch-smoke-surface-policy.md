---
type: Work Progress
title: Hotpatch smoke demo owner boundary enters surface policy
tags:
  - fret
  - hotpatch
  - surface-policy
  - model-owner
timestamp: 2026-07-07T13:53:14Z
---

# Summary

Promoted the `hotpatch_smoke_demo` owner-boundary source test into the global surface policy gate.
The demo remains a dev-only hotpatch maintainer harness, but event/command model writes now have a
repo-level guard that keeps them behind `HotpatchSmokeModelOwner`.

# Changed Files

- `tools/check_surface_policy.py`: classifies `apps/fret-demo/src/bin/hotpatch_smoke_demo.rs` as an
  internal harness, lists its explicit raw runtime seams, and adds
  `internal_harness-hotpatch-smoke-owner-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage proving direct `models_mut().update*`
  / `ModelStore::update*` bypasses are rejected while owner-routed writes are allowed.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples hotpatch_smoke_demo_routes_model_writes_through_owner --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
