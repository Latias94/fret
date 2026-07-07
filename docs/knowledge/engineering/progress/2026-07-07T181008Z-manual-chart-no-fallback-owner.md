---
type: Work Progress
title: Manual chart fallback owner removed
timestamp: 2026-07-07T18:10:08Z
tags:
  - fret-examples
  - surface-policy
  - chart
  - manual-surfaces
status: verified
---

# Summary

Removed the dead generic manual-chart surface fallback from `tools/check_surface_policy.py`.
Every filename in `MANUAL_CHART_DEMO_FILENAMES` now needs an explicit owner/reason branch in
`_fret_examples_manual_chart_surface`; otherwise policy construction fails fast.

# Truth

- The fallback could silently assign future manual chart demos an `examples-chart-*` owner with a
  generic retained-chart reason.
- The fail-fast branch keeps grouped fallback audits honest by requiring owner-specific evidence for
  any new manual chart surface.
- No runtime source changed; this slice only makes the advanced/manual surface classification more
  auditable.

# Artifacts

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_manual_chart_demo_classification_requires_explicit_owner`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
- `rg -n 'owner=f"examples-chart|manual retained-chart demo runner' tools/check_surface_policy.py` returned no matches.

# Notes

- This follows the earlier explicit-owner policy slices for chart demo, plot3d, streaming import,
  and custom-effect reference surfaces.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
