---
type: Work Progress
title: Examples surface helpers require explicit owners
timestamp: 2026-07-07T18:20:02Z
tags:
  - fret-examples
  - surface-policy
  - owner-evidence
status: verified
---

# Summary

Removed the remaining filename-derived owner fallback from the fret-examples classified-surface
helpers in `tools/check_surface_policy.py`.

# Truth

- `_fret_examples_advanced_surface(...)`, `_fret_examples_comparison_surface(...)`,
  `_fret_examples_internal_harness(...)`, and `_fret_examples_renderer_lab_surface(...)` now require
  keyword-only `owner` parameters.
- Existing call sites were already explicit, so this change only removes the stale fallback path and
  turns future omissions into immediate construction failures.
- No runtime source changed; this is a policy helper hardening slice.

# Artifacts

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_classified_helpers_require_explicit_owners`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
- `rg -n 'owner=owner or|owner: str \| None|owner=f"examples-|stem = .*replace' tools/check_surface_policy.py` returned no matches.
- AST audit reported `missing explicit owner calls: 0`.

# Notes

- This closes the generic examples helper fallback left after the grouped chart, custom-effect,
  streaming, and cookbook owner cleanups.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
