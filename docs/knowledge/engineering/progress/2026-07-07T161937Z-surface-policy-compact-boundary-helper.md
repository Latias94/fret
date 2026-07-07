---
type: Work Progress
title: Surface policy compact boundary helper
timestamp: 2026-07-07T16:19:37Z
tags:
  - source-policy
  - refactor
  - plot-binding
status: ready-for-commit
---

# Summary

The repeated compact-marker scanner functions for plot binding demos now share one table-driven
helper. Existing owner-specific rule names and marker sets remain intact.

# Outcome Truth

- Existing plot binding gates keep their current rule names and positive/negative fixture behavior.
- Adding the next manual chart demo gate now requires a new `CompactSourceBoundary` entry rather
  than another copy-pasted scanner function.
- Source-policy behavior remains covered by the full Python fixture suite and the real repository
  scan.

# Evidence

- `tools/check_surface_policy.py`: adds `CompactSourceBoundary`,
  `PLOT_DECLARATIVE_BINDING_BOUNDARIES`, `_scan_compact_source_boundary(...)`, and
  `_scan_plot_declarative_binding_boundaries(...)`.
- The old per-owner plot scanner functions were removed in favor of the shared helper.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
