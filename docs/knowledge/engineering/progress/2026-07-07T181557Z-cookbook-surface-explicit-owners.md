---
type: Work Progress
title: Cookbook surface owners are explicit
timestamp: 2026-07-07T18:15:57Z
tags:
  - fret-cookbook
  - surface-policy
  - renderer-lab
  - manual-surfaces
status: verified
---

# Summary

Changed the cookbook classified-surface helpers in `tools/check_surface_policy.py` so advanced and
renderer-lab cookbook surfaces must pass an explicit `owner`.

# Truth

- `_cookbook_advanced_surface(...)` and `_cookbook_renderer_lab_surface(...)` now require
  keyword-only `owner` parameters.
- The seven existing cookbook classified surfaces keep their previous owner strings, but the owner
  evidence now lives at each call site instead of being derived from filenames inside the helper.
- No runtime source changed; this is a surface-policy evidence tightening slice.

# Artifacts

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_cookbook_classified_surfaces_require_explicit_owners`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
- `rg -n 'owner = f"cookbook-' tools/check_surface_policy.py` returned no matches.

# Notes

- This follows the same direction as the explicit-owner cleanup for chart, plot3d, streaming import,
  custom-effect reference, and manual chart fallback policy records.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
